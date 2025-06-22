use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use tokio::process::Command;

/// ターゲットOSに応じて適切なシェルと引数を返すヘルパー関数
/// A helper function that returns the appropriate shell and arguments depending on the target OS.
#[cfg(target_os = "windows")]
fn get_os_shell_command() -> (&'static str, &'static str) {
    ("cmd.exe", "/C")
}

/// ターゲットOSに応じて適切なシェルと引数を返すヘルパー関数
/// A helper function that returns the appropriate shell and arguments depending on the target OS.
#[cfg(not(target_os = "windows"))] // Windows以外のOS (Linux, macOSなど)
fn get_os_shell_command() -> (&'static str, &'static str) {
    ("sh", "-c") // Unix系OSでは'sh -c'が一般的
}

pub async fn execute_command(command_str: &str, profile_id: &str, job_type: &str) -> Result<()> {
    info!(
        "Executing {} command for profile '{}': {}",
        job_type, profile_id, command_str
    );

    // OSに応じたシェルと引数を取得
    // Get shell and arguments according to OS
    let (shell, arg) = get_os_shell_command();

    let output = Command::new(shell) // For example, "cmd.exe" for Windows.
        .arg(arg) // For example, "/C" for Windows.
        .arg(command_str) // User-defined command string
        .output()
        .await
        .with_context(|| {
            format!(
                "Failed to execute {} command for profile '{}'",
                job_type, profile_id
            )
        })?;

    let stdout_trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr_trimmed = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout_trimmed.is_empty() {
        debug!(
            "{} command Stdout for profile '{}': {}",
            job_type, profile_id, stdout_trimmed
        );
    } else {
        debug!(
            "{} command Stdout for profile '{}' was empty.",
            job_type, profile_id
        );
    }

    if !stderr_trimmed.is_empty() {
        warn!(
            "{} command Stderr for profile '{}': {}",
            job_type, profile_id, stderr_trimmed
        );
    } else {
        debug!(
            "{} command Stderr for profile '{}' was empty.",
            job_type, profile_id
        );
    }

    if !output.status.success() {
        error!(
            "{} command failed for profile '{}'. Code: {:?}",
            job_type,
            profile_id,
            output.status.code()
        );
        return Err(anyhow::anyhow!(
            "{} command failed for profile '{}'",
            job_type,
            profile_id
        ));
    } else {
        info!(
            "{} command completed successfully for profile '{}'.",
            job_type, profile_id
        );
    }
    Ok(())
}
