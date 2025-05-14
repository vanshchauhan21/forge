use std::sync::Arc;

use colored::Colorize;
use forge_api::{Update, API};
use forge_tracker::{EventKind, VERSION};
use update_informer::{registry, Check, Version};

use crate::TRACKER;

/// Displays a formatted error message for manual update
async fn handle_update_error<S: AsRef<str>>(error: S) {
    println!(
        "{} Could not update Forge automatically.",
        "Update Failed:".bold().red()
    );
    println!("{} Run this command:", "Manual Update:".bold().yellow());
    println!("   {}", "npm i update -g @antinomyhq/forge".bold().cyan());
    let _ = send_update_failure_event(error.as_ref()).await;
}

/// Runs npm update in the background, failing silently
async fn execute_update_command(api: Arc<impl API>) {
    // Spawn a new task that won't block the main application
    let output = api
        .execute_shell_command("npm i update -g @antinomyhq/forge", api.environment().cwd)
        .await;

    match output {
        Err(err) => {
            // Send an event to the tracker on failure
            // We don't need to handle this result since we're failing silently
            handle_update_error(err.to_string()).await;
        }
        Ok(output) => {
            if output.stderr.is_empty() {
                let answer = inquire::Confirm::new(
                    "You need to close forge to complete update. Do you want to close it now?",
                )
                .with_default(true)
                .with_error_message("Invalid response!")
                .prompt();
                if answer.unwrap_or_default() {
                    std::process::exit(0);
                }
            } else {
                handle_update_error(format!("Auto update failed: {}", output.stderr)).await;
            }
        }
    }
}

async fn confirm_update(version: Version) -> bool {
    let answer = inquire::Confirm::new(&format!(
        "Confirm upgrade from {} -> {} (latest)?",
        VERSION.to_string().bold().white(),
        version.to_string().bold().white()
    ))
    .with_default(true)
    .with_error_message("Invalid response!")
    .prompt();

    answer.unwrap_or(false)
}

/// Checks if there is an update available
pub async fn on_update(api: Arc<impl API>, update: Option<&Update>) {
    let update = update.cloned().unwrap_or_default();
    let frequency = update.frequency.unwrap_or_default();
    let auto_update = update.auto_update.unwrap_or_default();

    // Check if version is development version, in which case we skip the update
    // check
    if VERSION.contains("dev") || VERSION == "0.1.0" {
        // Skip update for development version 0.1.0
        return;
    }

    let informer = update_informer::new(registry::Npm, "@antinomyhq/forge", VERSION)
        .interval(frequency.into());

    if let Some(version) = informer.check_version().ok().flatten() {
        if auto_update || confirm_update(version).await {
            execute_update_command(api).await;
        }
    }
}

/// Sends an event to the tracker when an update fails
async fn send_update_failure_event(error_msg: &str) -> anyhow::Result<()> {
    // Ignore the result since we are failing silently
    // This is safe because we're using a static tracker with 'static lifetime
    let _ = TRACKER
        .dispatch(EventKind::Error(error_msg.to_string()))
        .await;

    // Always return Ok since we want to fail silently
    Ok(())
}
