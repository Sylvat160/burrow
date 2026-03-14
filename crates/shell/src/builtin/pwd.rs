use super::*;
use crate::types::ShellError;
use std::io::Write;

pub struct Pwd;

impl BuiltIn for Pwd {
    fn execute(
        &self,
        _args: &[String],
        stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> Result<Action, ShellError> {
        let cwd = std::env::current_dir()?;
        writeln!(stdout, "{}", cwd.display())?;
        Ok(Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_prints_current_directory() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Pwd.execute(&[], &mut stdout, &mut stderr).unwrap();

        let out = String::from_utf8(stdout).unwrap();
        let expected = std::env::current_dir().unwrap();
        assert_eq!(out.trim(), expected.to_str().unwrap());
    }

    #[test]
    fn test_pwd_output_ends_with_newline() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Pwd.execute(&[], &mut stdout, &mut stderr).unwrap();

        assert!(stdout.ends_with(b"\n"));
        assert!(stderr.is_empty());
    }
}
