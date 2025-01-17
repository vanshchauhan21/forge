use std::io;

use colored::Colorize;
use forge_domain::Environment;

use crate::CONSOLE;

pub fn display_info(env: &Environment) -> io::Result<()> {
    CONSOLE.newline()?;
    CONSOLE.writeln(format!("{} {}", "OS:".dimmed(), env.os))?;
    CONSOLE.writeln(format!("{} {}", "Working Directory:".dimmed(), env.cwd))?;
    CONSOLE.writeln(format!("{} {}", "Shell:".dimmed(), env.shell))?;
    if let Some(home) = &env.home {
        CONSOLE.writeln(format!("{} {}", "Home Directory:".dimmed(), home))?;
    }
    CONSOLE.writeln(format!("{} {}", "File Count:".dimmed(), env.files.len()))?;
    CONSOLE.newline()?;
    CONSOLE.writeln(format!(
        "{} {}",
        "Primary Model:".dimmed(),
        env.large_model_id
    ))?;
    CONSOLE.writeln(format!(
        "{} {}",
        "Secondary Model:".dimmed(),
        env.small_model_id
    ))?;
    CONSOLE.newline()?;
    Ok(())
}
