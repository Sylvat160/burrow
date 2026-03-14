use rustyline::completion::{Completer, Pair};
use rustyline::{Context, Helper, Highlighter, Hinter, Validator};
use std::env;
use std::fs;

#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct ShellHelper {
    commands: Vec<String>,
}

impl ShellHelper {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub fn collect_executables() -> Vec<String> {
    let path = env::var("PATH").unwrap_or_default();
    let mut names: Vec<String> = Vec::new();

    for dir in path.split(":") {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() || file_type.is_symlink() {
                        if let Some(name) = entry.file_name().to_str() {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    names.sort();
    names.dedup();
    names
}

fn complete_path(prefix: &str) -> Vec<Pair> {
    let (dir, name_prefix) = match prefix.rfind("/") {
        Some(i) => (&prefix[..=i], &prefix[i + 1..]),
        None => ("./", prefix),
    };

    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };

    let mut matches: Vec<Pair> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            if !name.starts_with(name_prefix) {
                return None;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let suffix = if is_dir { "/" } else { " " };
            Some(Pair {
                display: format!("{}{}", name, suffix),
                replacement: format!("{}{}", &name[name_prefix.len()..], suffix),
            })
        })
        .collect();

    matches.sort_by(|a, b| a.display.cmp(&b.display));
    matches
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let word_start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let prefix = &line[word_start..pos];
        let is_argument = !line[..word_start].trim().is_empty();

        if is_argument {
            Ok((pos, complete_path(prefix)))
        } else {
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: format!("{} ", &cmd[prefix.len()..].to_string()),
                })
                .collect();

            Ok((pos, matches))
        }
    }
}
