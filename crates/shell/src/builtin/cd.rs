use super::{Action, BuiltIn};
use crate::types::ShellError;
use std::io::Write;

pub struct Cd;

impl BuiltIn for Cd {
    fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> Result<Action, ShellError> {
        let target = match args.first().map(|s| s.as_str()) {
            None | Some("~") => std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
            Some("-") => match std::env::var("OLDPWD") {
                Ok(p) => {
                    writeln!(stdout, "{}", p)?;
                    p
                }
                Err(_) => {
                    writeln!(stderr, "cd: OLDPWD is not set")?;
                    return Ok(Action::Continue);
                }
            },
            Some(p) => p.to_string(),
        };
        let old = std::env::current_dir().ok();

        match std::env::set_current_dir(&target) {
            Ok(_) => {
                if let Some(old) = old {
                    unsafe {
                        std::env::set_var("OLDPWD", old);
                    }
                }
                let new = std::env::current_dir()?;
                unsafe {
                    std::env::set_var("PWD", &new);
                }
            }
            Err(e) => {
                use std::io::ErrorKind;
                let msg = match e.kind() {
                    ErrorKind::NotFound => "cd: no such file or directory",
                    ErrorKind::PermissionDenied => "cd: permission denied",
                    _ => "cd: unknown error",
                };
                writeln!(stderr, "cd: {}: {}", target, msg)?;
            }
        }
        return Ok(Action::Continue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_dash_prints_path() {
        unsafe {
            std::env::set_var("OLDPWD", "/tmp");
        }
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();

        Cd.execute(&["-".into()], &mut stdout, &mut stderr).unwrap();
        // Assert
        assert_eq!(stdout, b"/tmp\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_cd_missing_dir_writes_to_stderr() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Cd.execute(&["/nonexistent_dir_xyz".into()], &mut stdout, &mut stderr)
            .unwrap();

        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }
}
