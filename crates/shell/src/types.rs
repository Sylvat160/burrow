use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Command {
    program: String,
    args: Vec<String>,
    redirections: Vec<Redirection>,
}

impl Command {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            redirections: Vec::new(),
        }
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn redirections(&self) -> &[Redirection] {
        &self.redirections
    }

    pub fn add_redirection(&mut self, redirection: Redirection) {
        self.redirections.push(redirection);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    fd: i32,
    target: String,
    mode: RedirectionMode,
}

impl Redirection {
    pub fn new(fd: i32, mode: RedirectionMode) -> Self {
        Self {
            fd,
            target: String::new(),
            mode,
        }
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn mode(&self) -> &RedirectionMode {
        &self.mode
    }

    pub fn add_target(&mut self, target: String) {
        self.target = target;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionMode {
    Input,
    Output,
    Append,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    commands: Vec<Command>,
}

impl Pipeline {
    pub fn new(commands: Vec<Command>) -> Self {
        Self { commands }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    Redirect(Redirection),
}

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("{0}: command not found")]
    CommandNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("syntax error: {0}")]
    SyntaxError(String),
}
