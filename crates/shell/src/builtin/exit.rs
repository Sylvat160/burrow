use super::*;
use std::io::Write;

pub struct Exit;

impl BuiltIn for Exit {
    fn execute(
        &self,
        _args: &[String],
        _stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> Result<Action, ShellError> {
        Ok(Action::Exit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_exit() {
        let mut stdout = Cursor::new(Vec::new());
        let mut stderr = Cursor::new(Vec::new());

        let exit = Exit;
        exit.execute(&[], &mut stdout, &mut stderr).unwrap();

        assert!(stdout.into_inner().is_empty());
        assert!(stderr.into_inner().is_empty());
    }
}
