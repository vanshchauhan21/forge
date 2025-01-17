use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use forge_domain::{NamedTool, ToolCallService, ToolDescription, ToolName};
use forge_tool_macros::ToolDescription;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ShellInput {
    /// The shell command to execute.
    pub command: String,
    /// The working directory where the command should be executed.
    pub cwd: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ShellOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    pub success: bool,
}

/// Execute shell commands with safety checks and validation. This tool provides
/// controlled access to system shell commands while preventing dangerous
/// operations through a comprehensive blacklist and validation system.
/// The tool also enforces a timeout to prevent long-running commands from
/// blocking the system.
#[derive(ToolDescription)]
pub struct Shell {
    blacklist: HashSet<String>,
    timeout_secs: u64,
}

impl Default for Shell {
    fn default() -> Self {
        let mut blacklist = HashSet::new();
        // File System Destruction Commands
        blacklist.insert("rm".to_string());
        blacklist.insert("rmdir".to_string());
        blacklist.insert("del".to_string());

        // Disk/Filesystem Commands
        blacklist.insert("format".to_string());
        blacklist.insert("mkfs".to_string());
        blacklist.insert("dd".to_string());

        // Permission/Ownership Commands
        blacklist.insert("chmod".to_string());
        blacklist.insert("chown".to_string());

        // Privilege Escalation Commands
        blacklist.insert("sudo".to_string());
        blacklist.insert("su".to_string());

        // Code Execution Commands
        blacklist.insert("exec".to_string());
        blacklist.insert("eval".to_string());

        // System Communication Commands
        blacklist.insert("write".to_string());
        blacklist.insert("wall".to_string());

        // System Control Commands
        blacklist.insert("shutdown".to_string());
        blacklist.insert("reboot".to_string());
        blacklist.insert("init".to_string());

        Shell { blacklist, timeout_secs: 30 }
    }
}

impl Shell {
    fn validate_command(&self, command: &str) -> Result<(), String> {
        let base_command = command
            .split_whitespace()
            .next()
            .ok_or_else(|| "Empty command".to_string())?;

        if self.blacklist.contains(base_command) {
            return Err(format!("Command '{}' is not allowed", base_command));
        }

        Ok(())
    }

    async fn execute_command(&self, command: &str, cwd: PathBuf) -> Result<ShellOutput, String> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };

        cmd.current_dir(cwd);

        let timeout_duration = Duration::from_secs(self.timeout_secs);
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => result.map_err(|e| e.to_string())?,
            Err(_) => {
                return Err(format!(
                    "Command '{}' timed out after {} seconds",
                    command, self.timeout_secs
                ))
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ShellOutput {
            stdout: if stdout.is_empty() {
                None
            } else {
                Some(stdout)
            },
            stderr: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
            success: output.status.success(),
        })
    }
}

impl NamedTool for Shell {
    fn tool_name(&self) -> ToolName {
        ToolName::new("tool_forge_process_shell")
    }
}

#[async_trait::async_trait]
impl ToolCallService for Shell {
    type Input = ShellInput;

    async fn call(&self, input: Self::Input) -> Result<String, String> {
        self.validate_command(&input.command)?;
        let output = self.execute_command(&input.command, input.cwd).await?;

        // Return error if stderr is present
        if let Some(stderr) = output.stderr {
            return Err(stderr);
        }

        // Handle stdout
        if let Some(stdout) = output.stdout {
            Ok(stdout)
        } else if output.success {
            Ok("Command executed successfully with no output.".to_string())
        } else {
            Ok("Command failed with no output.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use pretty_assertions::assert_eq;

    use super::*;

    /// Platform-specific error message patterns for command not found errors
    #[cfg(target_os = "windows")]
    const COMMAND_NOT_FOUND_PATTERNS: [&str; 2] = [
        "is not recognized",             // cmd.exe
        "'non_existent_command' is not", // PowerShell
    ];

    #[cfg(target_family = "unix")]
    const COMMAND_NOT_FOUND_PATTERNS: [&str; 3] = [
        "command not found",               // bash/sh
        "non_existent_command: not found", // bash/sh (Alternative Unix error)
        "No such file or directory",       // Alternative Unix error
    ];

    #[tokio::test]
    async fn test_shell_echo() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput {
                command: "echo 'Hello, World!'".to_string(),
                cwd: env::current_dir().unwrap(),
            })
            .await
            .unwrap();

        assert!(result.contains("Hello, World!"));
        assert!(!result.contains("Error:"));
    }

    #[tokio::test]
    async fn test_shell_with_working_directory() {
        let shell = Shell::default();
        let temp_dir = fs::canonicalize(env::temp_dir()).unwrap();

        let result = shell
            .call(ShellInput {
                command: if cfg!(target_os = "windows") {
                    "cd".to_string()
                } else {
                    "pwd".to_string()
                },
                cwd: temp_dir.clone(),
            })
            .await
            .unwrap();

        let output_path = PathBuf::from(result.trim());
        let actual_path = match fs::canonicalize(output_path.clone()) {
            Ok(path) => path,
            Err(_) => output_path,
        };
        let expected_path = temp_dir.as_path();

        assert_eq!(
            actual_path, expected_path,
            "\nExpected path: {:?}\nActual path: {:?}",
            expected_path, actual_path
        );
        assert!(!result.contains("Error"));
    }

    #[tokio::test]
    async fn test_shell_invalid_command() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput {
                command: "non_existent_command".to_string(),
                cwd: env::current_dir().unwrap(),
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();

        // Check if any of the platform-specific patterns match
        let matches_pattern = COMMAND_NOT_FOUND_PATTERNS
            .iter()
            .any(|&pattern| err.contains(pattern));

        assert!(
            matches_pattern,
            "Error message '{}' did not match any expected patterns for this platform: {:?}",
            err, COMMAND_NOT_FOUND_PATTERNS
        );
    }

    #[tokio::test]
    async fn test_shell_blacklisted_command() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput {
                command: "rm -rf /".to_string(),
                cwd: env::current_dir().unwrap(),
            })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_shell_empty_command() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput { command: "".to_string(), cwd: env::current_dir().unwrap() })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty command"));
    }

    #[tokio::test]
    async fn test_shell_pwd() {
        let shell = Shell::default();
        let current_dir = fs::canonicalize(env::current_dir().unwrap()).unwrap();
        let result = shell
            .call(ShellInput {
                command: if cfg!(target_os = "windows") {
                    "cd".to_string()
                } else {
                    "pwd".to_string()
                },
                cwd: current_dir.clone(),
            })
            .await
            .unwrap();

        let output_path = PathBuf::from(result.trim());
        let actual_path = match fs::canonicalize(output_path.clone()) {
            Ok(path) => path,
            Err(_) => output_path,
        };
        assert_eq!(actual_path, current_dir);
        assert!(!result.contains("Error:"));
    }

    #[tokio::test]
    async fn test_shell_multiple_commands() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput {
                command: "echo 'first' && echo 'second'".to_string(),
                cwd: env::current_dir().unwrap(),
            })
            .await
            .unwrap();

        assert!(result.contains("first"));
        assert!(result.contains("second"));
        assert!(!result.contains("Error:"));
    }

    #[tokio::test]
    async fn test_shell_with_environment_variables() {
        let shell = Shell::default();
        let result = shell
            .call(ShellInput {
                command: "echo $PATH".to_string(),
                cwd: env::current_dir().unwrap(),
            })
            .await
            .unwrap();

        assert!(!result.is_empty());
        assert!(!result.contains("Error:"));
    }

    #[tokio::test(start_paused = true)]
    async fn test_shell_command_timeout() {
        let shell = Shell::default();
        let handle = tokio::spawn(async move {
            shell
                .call(ShellInput {
                    command: if cfg!(target_os = "windows") {
                        "timeout /t 60".to_string()
                    } else {
                        "sleep 60".to_string()
                    },
                    cwd: env::current_dir().unwrap(),
                })
                .await
        });

        // Advance time past the timeout
        tokio::time::advance(Duration::from_secs(30)).await;

        let result = handle.await.unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out after 30 seconds"));
    }

    #[test]
    fn test_description() {
        assert!(Shell::default().description().len() > 100)
    }
}
