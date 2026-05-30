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
        assert!(res.is_ok());
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_toggle_empty_or_own_pid() {
        let _guard = EnvGuard::new("own-pid");
        let pid_path = paths::pid_file(Mode::Agenda);

        // Empty file
        fs::write(&pid_path, "").unwrap();
        assert!(toggle_existing_instance(Mode::Agenda).is_ok());
        assert!(!pid_path.exists());

        // Own pid
        fs::write(&pid_path, std::process::id().to_string()).unwrap();
        assert!(pid_path.exists());
        assert!(toggle_existing_instance(Mode::Agenda).is_ok());
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_toggle_nonexistent_pid() {
        let _guard = EnvGuard::new("nonexistent-pid");
        let pid_path = paths::pid_file(Mode::Agenda);

        // A PID that definitely doesn't exist
        fs::write(&pid_path, "999999").unwrap();
        assert!(pid_path.exists());
        assert!(toggle_existing_instance(Mode::Agenda).is_ok());
        assert!(!pid_path.exists());
    }
}
