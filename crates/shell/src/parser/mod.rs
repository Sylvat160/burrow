use crate::types::{Command, Pipeline, Redirection, ShellError, Token};

mod lexer;

pub fn parse(input: &str) -> Result<Option<Pipeline>, ShellError> {
    let tokens = lexer::tokenize(input)?;

    if tokens.is_empty() {
        Ok(None)
    } else {
        let commands: Vec<Command> = tokens
            .split(|t| matches!(t, Token::Pipe))
            .map(|group| {
                let mut words: Vec<String> = Vec::new();
                let mut redirections: Vec<Redirection> = Vec::new();
                let mut i = 0;

                while i < group.len() {
                    match &group[i] {
                        Token::Word(s) => {
                            words.push(s.clone());
                        }
                        Token::Redirect(redir) => {
                            if i + 1 >= group.len() {
                                return Err(ShellError::Parse(
                                    "missing redirection target".to_string(),
                                ));
                            }
                            let next = &group[i + 1];
                            match next {
                                Token::Word(s) => {
                                    let mut redirection =
                                        Redirection::new(redir.fd(), redir.mode().clone());
                                    redirection.add_target(s.clone());
                                    redirections.push(redirection);
                                    i += 1;
                                }
                                _ => {
                                    return Err(ShellError::Parse(
                                        "unexpected token after redirection".to_string(),
                                    ));
                                }
                            }
                        }
                        Token::Pipe => {
                            return Err(ShellError::Parse("unexpected pipe".to_string()));
                        }
                    }
                    i += 1;
                }

                let program = words[0].clone();
                let args = words[1..].to_vec();
                let mut command = Command::new(program, args);
                for r in redirections {
                    command.add_redirection(r);
                }

                Ok(command)
            })
            .collect::<Result<Vec<Command>, ShellError>>()?;
        let pipeline = Pipeline::new(commands);
        Ok(Some(pipeline))
    }
}
