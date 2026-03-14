use super::*;
use std::io::Write;

pub struct Export;

impl BuiltIn for Export {
    fn execute(
        &self,
        args: &[String],
        _stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> Result<Action, ShellError> {
        for arg in args {
            if let Some((key, value)) = arg.split_once('=') {
                unsafe {
                    std::env::set_var(key, value);
                }
            }
        }
        Ok(Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_one_variable() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Export
            .execute(&["BURROW_TEST_A=hello".into()], &mut stdout, &mut stderr)
            .unwrap();

        assert_eq!(std::env::var("BURROW_TEST_A").unwrap(), "hello");
    }

    #[test]
    fn test_export_two_variables() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Export
            .execute(
                &["BURROW_TEST_B=foo".into(), "BURROW_TEST_C=bar".into()],
                &mut stdout,
                &mut stderr,
            )
            .unwrap();

        assert_eq!(std::env::var("BURROW_TEST_B").unwrap(), "foo");
        assert_eq!(std::env::var("BURROW_TEST_C").unwrap(), "bar");
    }

    #[test]
    fn test_export_value_with_equals_sign() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        Export
            .execute(&["BURROW_TEST_D=a=b=c".into()], &mut stdout, &mut stderr)
            .unwrap();

        // split_once ensures only the first '=' is the delimiter
        assert_eq!(std::env::var("BURROW_TEST_D").unwrap(), "a=b=c");
    }
}
