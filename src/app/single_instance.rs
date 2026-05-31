use crate::calendar::model::Mode;
use crate::storage::paths;
use std::fs;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceStatus {
    StartNew,
    TerminatedExisting,
}

pub fn toggle_existing_instance(mode: Mode) -> Result<InstanceStatus, String> {
    let file = paths::pid_file(mode);
    let Ok(raw) = fs::read_to_string(&file) else {
        return Ok(InstanceStatus::StartNew);
    };

    let Some(pid) = parse_pid(&raw) else {
        let _ = fs::remove_file(&file);
        return Ok(InstanceStatus::StartNew);
    };

    if pid == std::process::id() {
        let _ = fs::remove_file(&file);
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
    use std::env;
    use std::path::PathBuf;

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        original_runtime: Option<std::ffi::OsString>,
        temp_dir: PathBuf,
    }

    impl EnvGuard {
        fn new(name: &str) -> Self {
            let lock = crate::test_env::ENV_LOCK
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let original_runtime = env::var_os("XDG_RUNTIME_DIR");
            let temp_dir = env::temp_dir().join(format!("gcal-test-runtime-{}", name));
            let _ = fs::remove_dir_all(&temp_dir);
            let _ = fs::create_dir_all(&temp_dir);
            unsafe {
                env::set_var("XDG_RUNTIME_DIR", &temp_dir);
            }
            Self {
                _lock: lock,
                original_runtime,
                temp_dir,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(v) = &self.original_runtime {
                    env::set_var("XDG_RUNTIME_DIR", v);
                } else {
                    env::remove_var("XDG_RUNTIME_DIR");
                }
            }
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }

    #[test]
    fn test_toggle_no_pid_file() {
        let _guard = EnvGuard::new("no-pid");
        let pid_path = paths::pid_file(Mode::Agenda);
        assert!(!pid_path.exists());

        let res = toggle_existing_instance(Mode::Agenda);
        assert_eq!(res.unwrap(), InstanceStatus::StartNew);
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_toggle_empty_or_own_pid() {
        let _guard = EnvGuard::new("own-pid");
        let pid_path = paths::pid_file(Mode::Agenda);

        // Empty file
        fs::write(&pid_path, "").unwrap();
        assert_eq!(
            toggle_existing_instance(Mode::Agenda).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_path.exists());

        // Own pid
        fs::write(&pid_path, std::process::id().to_string()).unwrap();
        assert!(pid_path.exists());
        assert_eq!(
            toggle_existing_instance(Mode::Agenda).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_toggle_invalid_pid() {
        let _guard = EnvGuard::new("invalid-pid");
        let pid_path = paths::pid_file(Mode::Agenda);

        fs::write(&pid_path, "-1").unwrap();
        assert_eq!(
            toggle_existing_instance(Mode::Agenda).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_toggle_nonexistent_pid() {
        let _guard = EnvGuard::new("nonexistent-pid");
        let pid_path = paths::pid_file(Mode::Agenda);

        // A PID that definitely doesn't exist
        fs::write(&pid_path, "999999").unwrap();
        assert!(pid_path.exists());
        assert_eq!(
            toggle_existing_instance(Mode::Agenda).unwrap(),
            InstanceStatus::StartNew
        );
        assert!(!pid_path.exists());
    }
}
