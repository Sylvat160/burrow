use crate::types::ShellError;

pub mod cd;
pub mod echo;
pub mod exit;
pub mod export;
pub mod history;
pub mod pwd;
pub mod type_cmd;

#[derive(Debug, PartialEq)]
pub enum Action {
    Continue,
    Exit,
}

pub trait BuiltIn {
    fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<Action, ShellError>;
}
