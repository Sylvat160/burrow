use super::BuiltIn;
use std::io::Write;

pub struct Echo;

impl BuiltIn for Echo {
    fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> Result<super::Action, super::ShellError> {
        writeln!(stdout, "{}", args.join(" "))?;
        Ok(super::Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_prints_args() {
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();

        Echo.execute(&["hello".into(), "world".into()], &mut stdout, &mut stderr)
            .unwrap();

        assert_eq!(stdout, b"hello world\n");
        assert!(stderr.is_empty());
    }
}
