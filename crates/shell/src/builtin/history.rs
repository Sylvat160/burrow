use super::*;
use std::io::Write;

pub struct History {
    commands: Vec<String>,
    write_offset: std::cell::Cell<usize>,
}

impl History {
    pub fn new(commands: Vec<String>, file_offset: usize) -> Self {
        History {
            commands,
            write_offset: std::cell::Cell::new(file_offset),
        }
    }
}
impl BuiltIn for History {
    fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn std::io::Write,
        _stderr: &mut dyn std::io::Write,
    ) -> Result<Action, ShellError> {
        match args.first().map(|s| s.as_str()) {
            Some("-r") => {
                // silently succeed - entries loaded at startup via load history
            }
            Some("-w") => {
                if let Some(path) = args.get(1) {
                    let content = self.commands.join("\n") + "\n";
                    std::fs::write(path, content)?;
                };
            }
            Some("-a") => {
                if let Some(path) = args.get(1) {
                    let start = self.write_offset.get();
                    let mut file = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)?;
                    for entry in &self.commands[start..] {
                        writeln!(file, "{}", entry)?;
                    }
                    self.write_offset.set(self.commands.len());
                };
            }
            Some(n_str) => {
                let n: usize = n_str.parse().map_err(|_| {
                    crate::types::ShellError::InvalidArgument(format!(
                        "history: {}: numeric agent required",
                        n_str
                    ))
                })?;
                let start = self.commands.len().saturating_sub(n);
                for (index, command) in self.commands[start..].iter().enumerate() {
                    writeln!(stdout, "{:>4}  {}", start + index + 1, command)?;
                }
            }
            None => {
                for (index, command) in self.commands.iter().enumerate() {
                    writeln!(stdout, "{:>4}  {}", index + 1, command)?;
                }
            }
        }
        Ok(Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_history() -> History {
        History::new(vec!["ls".into(), "cd /tmp".into(), "echo hello".into()], 0)
    }

    #[test]
    fn test_history_no_args_prints_all() {
        let h = make_history();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        h.execute(&[], &mut stdout, &mut stderr).unwrap();

        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains("ls"));
        assert!(out.contains("cd /tmp"));
        assert!(out.contains("echo hello"));
        // first entry is numbered 1
        assert!(out.contains("   1  ls"));
    }

    #[test]
    fn test_history_n_prints_last_n() {
        let h = make_history();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        h.execute(&["2".into()], &mut stdout, &mut stderr).unwrap();

        let out = String::from_utf8(stdout).unwrap();
        // last 2 entries shown
        assert!(out.contains("cd /tmp"));
        assert!(out.contains("echo hello"));
        // first entry NOT shown
        assert!(!out.contains("ls\n"));
    }

    #[test]
    fn test_history_invalid_n_returns_error() {
        let h = make_history();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let result = h.execute(&["abc".into()], &mut stdout, &mut stderr);

        assert!(result.is_err());
    }

    #[test]
    fn test_history_write_creates_file() {
        let h = make_history();
        let path = format!("/tmp/burrow_test_history_w_{}.txt", std::process::id());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        h.execute(&["-w".into(), path.clone()], &mut stdout, &mut stderr)
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "ls\ncd /tmp\necho hello\n");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_history_append_only_writes_new_entries() {
        // file_offset=1 means first entry was already saved, only append from index 1 onwards
        let h = History::new(vec!["ls".into(), "cd /tmp".into(), "echo hello".into()], 1);
        let path = format!("/tmp/burrow_test_history_a_{}.txt", std::process::id());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        h.execute(&["-a".into(), path.clone()], &mut stdout, &mut stderr)
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // only entries after offset 1 are written
        assert_eq!(content, "cd /tmp\necho hello\n");
        std::fs::remove_file(path).unwrap();
    }
}
