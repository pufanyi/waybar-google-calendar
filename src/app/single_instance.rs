use crate::calendar::model::Mode;
use crate::storage::paths;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceStatus {
    StartNew,
    TerminatedExisting,
}

pub fn toggle_existing_instance(mode: Mode) -> Result<InstanceStatus, String> {
    let file = paths::pid_file(mode);
    toggle_existing_instance_file(&file)
}

fn toggle_existing_instance_file(file: &Path) -> Result<InstanceStatus, String> {
    let Ok(raw) = fs::read_to_string(file) else {
        return Ok(InstanceStatus::StartNew);
    };

    let Some(pid) = parse_pid(&raw) else {
        let _ = fs::remove_file(file);
        return Ok(InstanceStatus::StartNew);
    };

    if pid == std::process::id() {
        let _ = fs::remove_file(file);
        return Ok(InstanceStatus::StartNew);
    }

    if process_exists(pid)? && process_looks_like_this_app(pid) && terminate_process(pid)? {
        return Ok(InstanceStatus::TerminatedExisting);
    }

    let _ = fs::remove_file(file);
    Ok(InstanceStatus::StartNew)
}

fn parse_pid(raw: &str) -> Option<u32> {
    raw.trim().parse::<u32>().ok().filter(|pid| *pid > 0)
}

fn process_exists(pid: u32) -> Result<bool, String> {
    send_signal(pid, 0, SignalPurpose::Inspect)
}

fn terminate_process(pid: u32) -> Result<bool, String> {
    send_signal(pid, libc::SIGTERM, SignalPurpose::Terminate)
}

#[derive(Debug, Clone, Copy)]
enum SignalPurpose {
    Inspect,
    Terminate,
}

#[cfg(unix)]
fn send_signal(pid: u32, signal: i32, purpose: SignalPurpose) -> Result<bool, String> {
    let pid = libc::pid_t::try_from(pid)
        .map_err(|_| format!("Existing pid {pid} does not fit this platform"))?;
    // SAFETY: `kill` is called with a checked platform pid_t and does not
    // dereference Rust pointers. Errors are read immediately from errno.
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        return Ok(true);
    }

    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) if matches!(purpose, SignalPurpose::Inspect) => Ok(true),
        _ => Err(match purpose {
            SignalPurpose::Inspect => format!("Could not inspect existing process {pid}: {error}"),
            SignalPurpose::Terminate => {
                format!("Could not terminate existing process {pid}: {error}")
            }
        }),
    }
}

#[cfg(not(unix))]
fn send_signal(_pid: u32, _signal: i32, _purpose: SignalPurpose) -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "linux")]
fn process_looks_like_this_app(pid: u32) -> bool {
    let Ok(current) = std::env::current_exe() else {
        return false;
    };
    let Ok(candidate) = fs::read_link(format!("/proc/{pid}/exe")) else {
        return false;
    };
    current.file_name() == candidate.file_name()
}

#[cfg(not(target_os = "linux"))]
fn process_looks_like_this_app(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempPidFile {
        path: PathBuf,
    }

    impl TempPidFile {
        fn new(name: &str) -> Self {
            let temp_dir = std::env::temp_dir().join(format!(
                "gcal-test-runtime-{}-{}",
                name,
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&temp_dir);
            fs::create_dir_all(&temp_dir).unwrap();
            Self {
                path: temp_dir.join("app.pid"),
            }
        }
    }

    impl Drop for TempPidFile {
        fn drop(&mut self) {
            if let Some(parent) = self.path.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }

    #[test]
    fn test_toggle_no_pid_file() {
        let pid_file = TempPidFile::new("no-pid");
        assert!(!pid_file.path.exists());

        let res = toggle_existing_instance_file(&pid_file.path);
        assert_eq!(res.unwrap(), InstanceStatus::StartNew);
        assert!(!pid_file.path.exists());
    }

    #[test]
    fn test_toggle_empty_or_own_pid() {
        let pid_file = TempPidFile::new("own-pid");

        // Empty file
        fs::write(&pid_file.path, "").unwrap();
        assert_eq!(
            toggle_existing_instance_file(&pid_file.path).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_file.path.exists());

        // Own pid
        fs::write(&pid_file.path, std::process::id().to_string()).unwrap();
        assert!(pid_file.path.exists());
        assert_eq!(
            toggle_existing_instance_file(&pid_file.path).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_file.path.exists());
    }

    #[test]
    fn test_toggle_invalid_pid() {
        let pid_file = TempPidFile::new("invalid-pid");

        fs::write(&pid_file.path, "-1").unwrap();
        assert_eq!(
            toggle_existing_instance_file(&pid_file.path).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_file.path.exists());
    }

    #[test]
    fn test_toggle_nonexistent_pid() {
        let pid_file = TempPidFile::new("nonexistent-pid");

        // A PID that definitely doesn't exist
        fs::write(&pid_file.path, "999999").unwrap();
        assert!(pid_file.path.exists());
        assert_eq!(
            toggle_existing_instance_file(&pid_file.path).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_file.path.exists());
    }
}
