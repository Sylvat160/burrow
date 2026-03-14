use crate::{
    builtin::{Action, BuiltIn},
    types::{Command, Redirection, RedirectionMode, ShellError},
    utils::find_in_path,
};
use std::{collections::HashMap, process::Stdio};
use std::{
    fs::{File, OpenOptions},
    io, path,
};

fn resolve_stdio(fd: i32, redirections: &[Redirection]) -> Result<Stdio, ShellError> {
    if let Some(redir) = redirections.iter().find(|r| r.fd() == fd) {
        match redir.mode() {
            RedirectionMode::Output => Ok(Stdio::from(File::create(redir.target())?)),
            RedirectionMode::Append => Ok(Stdio::from(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(redir.target())?,
            )),
            RedirectionMode::Input => Ok(Stdio::inherit()),
        }
    } else {
        Ok(Stdio::inherit())
    }
}

pub fn execute_command(
    command: &Command,
    builtins: &HashMap<String, Box<dyn BuiltIn>>,
    stdin_override: Option<os_pipe::PipeReader>,
    stdout_override: Option<os_pipe::PipeWriter>,
) -> Result<(Action, Option<std::process::Child>), ShellError> {
    let mut stdout: Box<dyn io::Write> =
        if let Some(redir) = command.redirections().iter().find(|r| r.fd() == 1) {
            match redir.mode() {
                RedirectionMode::Output => Box::new(File::create(redir.target())?),
                RedirectionMode::Append => Box::new(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(redir.target())?,
                ),
                RedirectionMode::Input => Box::new(io::stdout()),
            }
        } else {
            Box::new(io::stdout())
        };

    let mut stderr: Box<dyn io::Write> =
        if let Some(redir) = command.redirections().iter().find(|r| r.fd() == 2) {
            match redir.mode() {
                RedirectionMode::Output => Box::new(File::create(redir.target())?),
                RedirectionMode::Append => Box::new(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(redir.target())?,
                ),
                RedirectionMode::Input => Box::new(io::stderr()),
            }
        } else {
            Box::new(io::stderr())
        };

    let program = command.program();
    let args = command.args();

    let action = if let Some(builtin) = builtins.get(program) {
        if let Some(out) = stdout_override {
            stdout = Box::new(out);
        }
        builtin.execute(args, &mut stdout, &mut stderr)?
    } else if let Some(p) = find_in_path(program) {
        let bin = p.split(path::MAIN_SEPARATOR).last().unwrap();
        let stdout_stdio = if command.redirections().iter().any(|r| r.fd() == 1) {
            resolve_stdio(1, command.redirections())?
        } else if let Some(out) = stdout_override {
            out.into()
        } else {
            Stdio::inherit()
        };
        let stdin_stdio = if let Some(inp) = stdin_override {
            inp.into()
        } else {
            Stdio::inherit()
        };
        let stderr_stdio = resolve_stdio(2, command.redirections())?;
        let child = std::process::Command::new(bin)
            .args(args)
            .stdout(stdout_stdio)
            .stderr(stderr_stdio)
            .stdin(stdin_stdio)
            .spawn()?;
        return Ok((Action::Continue, Some(child)));
    } else {
        // return Err(ShellError::CommandNotFound(program.to_string()));
        writeln!(stderr, "{}: command not found", program)?;
        Action::Continue
    };

    Ok((action, None))
}
