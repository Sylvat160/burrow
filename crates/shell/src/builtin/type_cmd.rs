use super::*;
use crate::utils::find_in_path;

pub struct TypeCmd {
    builtin_names: Vec<String>,
}

impl TypeCmd {
    pub fn new(builtin_names: Vec<String>) -> Self {
        Self { builtin_names }
    }
}

impl BuiltIn for TypeCmd {
    fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<Action, ShellError> {
        for arg in args {
            if self.builtin_names.contains(arg) {
                writeln!(stdout, "{}: is a burrow builtin", arg)?;
            } else if let Some(path) = find_in_path(arg) {
                writeln!(stdout, "{}: is a shell command ({})", arg, path)?;
            } else {
                writeln!(stderr, "{}: command not found", arg)?;
            }
        }
        Ok(Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_type_cmd() -> TypeCmd {
        TypeCmd::new(vec!["cd".into(), "exit".into(), "export".into()])
    }

    #[test]
    fn test_type_recognizes_builtin() {
        let t = make_type_cmd();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        t.execute(&["cd".into()], &mut stdout, &mut stderr).unwrap();

        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains("cd: is a burrow builtin"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_type_finds_external_command() {
        let t = make_type_cmd();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        // "ls" is always present on unix systems
        t.execute(&["ls".into()], &mut stdout, &mut stderr).unwrap();

        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains("ls: is a shell command"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_type_unknown_command_writes_to_stderr() {
        let t = make_type_cmd();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        t.execute(
            &["definitely_not_a_real_command_xyz".into()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        assert!(stdout.is_empty());
        let err = String::from_utf8(stderr).unwrap();
        assert!(err.contains("command not found"));
    }

    #[test]
    fn test_type_multiple_args() {
        let t = make_type_cmd();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        t.execute(&["cd".into(), "ls".into()], &mut stdout, &mut stderr)
            .unwrap();

        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains("cd: is a burrow builtin"));
        assert!(out.contains("ls: is a shell command"));
    }
}
