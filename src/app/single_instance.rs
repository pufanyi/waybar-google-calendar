use crate::calendar::model::Mode;
use crate::storage::paths;
use std::fs;
use std::io;
use std::process::{Command, Stdio};

pub fn toggle_existing_instance(mode: Mode) -> Result<(), String> {
    let file = paths::pid_file(mode);
    let Ok(raw) = fs::read_to_string(&file) else {
        return Ok(());
    };

    let pid = raw.trim();
    if pid.is_empty() || pid == std::process::id().to_string() {
        let _ = fs::remove_file(&file);
        return Ok(());
    }

    if process_exists(pid)? {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        std::process::exit(0);
    }

    let _ = fs::remove_file(file);
    Ok(())
}

fn process_exists(pid: &str) -> Result<bool, String> {
    let status = Command::new("kill")
        .arg("-0")
        .arg(pid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err: io::Error| format!("Could not inspect existing process {pid}: {err}"))?;
    Ok(status.success())
}
