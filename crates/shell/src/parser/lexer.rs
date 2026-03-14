use crate::types::{Redirection, RedirectionMode, ShellError, Token};

#[derive(Debug, Clone, Copy)]
enum LexerState {
    Normal,
    SingleQuote,
    DoubleQuote,
    BackSlash(ReturnState),
    GreaterThan(i32),
    Variable(ReturnState),
}

#[derive(Debug, Copy, Clone)]
enum ReturnState {
    Normal,
    DoubleQuote,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut token = String::new();
    let mut state = LexerState::Normal;
    let mut var_name = String::new();

    let flush_token = |token: &mut String, tokens: &mut Vec<Token>| {
        if !token.is_empty() {
            tokens.push(Token::Word(token.clone()));
            token.clear();
        }
    };

    for c in input.trim().chars() {
        let mut reprocess = true;
        while reprocess {
            reprocess = false;
            match state {
                LexerState::Normal => {
                    if c == '\'' {
                        state = LexerState::SingleQuote;
                    } else if c == '"' {
                        state = LexerState::DoubleQuote;
                    } else if c == '\\' {
                        state = LexerState::BackSlash(ReturnState::Normal);
                    } else if c == '|' {
                        flush_token(&mut token, &mut tokens);
                        tokens.push(Token::Pipe);
                    } else if c == '>' {
                        if token.len() == 1 && matches!(token.as_str(), "0" | "1" | "2") {
                            state = LexerState::GreaterThan(token.parse().unwrap());
                            token = String::new();
                        } else {
                            flush_token(&mut token, &mut tokens);
                            state = LexerState::GreaterThan(1);
                        }
                    } else if c == '<' {
                        flush_token(&mut token, &mut tokens);
                        tokens.push(Token::Redirect(Redirection::new(0, RedirectionMode::Input)));
                    } else if c == '$' {
                        state = LexerState::Variable(ReturnState::Normal);
                    } else if c != ' ' {
                        token.push(c);
                    } else if !token.is_empty() {
                        flush_token(&mut token, &mut tokens);
                    }
                }
                LexerState::SingleQuote => {
                    if c == '\'' {
                        state = LexerState::Normal;
                    } else {
                        token.push(c);
                    }
                }
                LexerState::DoubleQuote => {
                    if c == '"' {
                        state = LexerState::Normal;
                    } else if c == '\\' {
                        state = LexerState::BackSlash(ReturnState::DoubleQuote);
                    } else if c == '$' {
                        state = LexerState::Variable(ReturnState::DoubleQuote);
                    } else {
                        token.push(c);
                    }
                }
                LexerState::BackSlash(context) => match context {
                    ReturnState::DoubleQuote => {
                        if matches!(c, '"' | '\\' | '$' | '`' | '\n') {
                            token.push(c);
                        } else {
                            token.push('\\');
                            token.push(c);
                        }
                        state = LexerState::DoubleQuote;
                    }
                    ReturnState::Normal => {
                        token.push(c);
                        state = LexerState::Normal;
                    }
                },

                LexerState::GreaterThan(fd) => {
                    if c == '>' {
                        tokens.push(Token::Redirect(Redirection::new(
                            fd,
                            RedirectionMode::Append,
                        )));
                    } else {
                        tokens.push(Token::Redirect(Redirection::new(
                            fd,
                            RedirectionMode::Output,
                        )));
                        reprocess = true;
                    }

                    state = LexerState::Normal;
                }

                LexerState::Variable(context) => {
                    if c.is_alphanumeric() || c == '_' {
                        var_name.push(c);
                    } else {
                        if var_name.is_empty() {
                            token.push('$');
                        } else {
                            token.push_str(&std::env::var(&var_name).unwrap_or_default());
                            var_name.clear();
                        }
                        state = match context {
                            ReturnState::DoubleQuote => LexerState::DoubleQuote,
                            ReturnState::Normal => LexerState::Normal,
                        };
                        reprocess = true;
                    }
                }
            }
        }
    }

    if matches!(state, LexerState::SingleQuote) {
        return Err(ShellError::Parse("unclose single quote".to_string()));
    }

    if matches!(state, LexerState::DoubleQuote) {
        return Err(ShellError::Parse("unclosed double quote".to_string()));
    }

    if let LexerState::GreaterThan(fd) = state {
        tokens.push(Token::Redirect(Redirection::new(
            fd,
            RedirectionMode::Output,
        )));
    }

    if let LexerState::Variable(_) = state {
        if var_name.is_empty() {
            token.push('$');
        } else {
            token.push_str(&std::env::var(&var_name).unwrap_or_default());
            var_name.clear();
        }
    }

    if !token.is_empty() {
        tokens.push(Token::Word(token));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_word() {
        let tokens = tokenize("hello").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("hello".to_string()));
    }

    #[test]
    fn tokenize_word_with_variable() {
        let tokens = tokenize("hello $USER").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("hello".to_string()));
        assert_eq!(
            tokens[1],
            Token::Word(std::env::var("USER").unwrap_or_default())
        );
    }

    #[test]
    fn tokenize_word_with_variable_and_space() {
        let tokens = tokenize("hello $USER world").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("hello".to_string()));
        assert_eq!(
            tokens[1],
            Token::Word(std::env::var("USER").unwrap_or_default())
        );
        assert_eq!(tokens[2], Token::Word("world".to_string()));
    }

    #[test]
    fn tokenize_word_with_variable_and_multiple_spaces() {
        let tokens = tokenize("hello $USER   world").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("hello".to_string()));
        assert_eq!(
            tokens[1],
            Token::Word(std::env::var("USER").unwrap_or_default())
        );
        assert_eq!(tokens[2], Token::Word("world".to_string()));
    }

    #[test]
    fn tokenize_single_quotes() {
        let tokens = tokenize("'hello $SYLVA'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("hello $SYLVA".to_string()));
    }
}
