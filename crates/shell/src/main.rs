#[allow(unused_imports)]
// use crate::parser::parse;
use anyhow::Result;
use burrow_shell::{
    builtin::{
        Action, BuiltIn, cd::Cd, echo::Echo, exit::Exit, export::Export, history::History,
        pwd::Pwd, type_cmd::TypeCmd,
    },
    completer::{ShellHelper, collect_executables},
    executor::execute_command,
    parser::parse,
};
use rustyline::CompletionType;
use rustyline::Editor;
use rustyline::config::Config;
use rustyline::{error::ReadlineError, history::History as RlHistory};
use std::collections::HashMap;
use std::io::Write;

fn main() -> Result<()> {
    // TODO: Uncomment the code below to pass the first stage

    let mut builtins: HashMap<String, Box<dyn BuiltIn>> = HashMap::new();
    builtins.insert("echo".to_string(), Box::new(Echo));
    builtins.insert(String::from("exit"), Box::new(Exit));
    builtins.insert(
        String::from("type"),
        Box::new(TypeCmd::new(vec![
            String::from("echo"),
            String::from("type"),
            String::from("exit"),
            String::from("export"),
            String::from("history"),
            String::from("pwd"),
            String::from("cd"),
        ])),
    );
    builtins.insert("export".to_string(), Box::new(Export));
    builtins.insert(String::from("pwd"), Box::new(Pwd));
    builtins.insert(String::from("cd"), Box::new(Cd));

    let hist_file = std::env::var("HISTFILE").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.myshell_history")
    });

    // let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    // let hist_file = format!("{home}/.myshell_history");

    let mut commands = collect_executables();
    let builtin_names = vec!["echo", "type", "exit", "export", "history"];
    for name in builtin_names {
        if !commands.contains(&name.to_string()) {
            commands.push(name.to_string());
        }
    }
    commands.sort();

    let helper = ShellHelper::new(commands);
    // let mut rl = Editor::new()?;
    let config = Config::builder()
        .completion_type(CompletionType::List)
        // .history_ignore_dups(true)?
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(helper));
    let _ = rl.load_history(&hist_file);
    let history_file_offset = rl.history().len();
    let mut history_write_offset = history_file_offset;

    loop {
        // print!("$ ");
        // io::stdout().flush()?;
        // let mut input = String::new();
        // let bytes_read = io::stdin().read_line(&mut input)?;

        // if bytes_read == 0 {
        //     break;
        // }

        match rl.readline("$ ") {
            Ok(input) => {
                let _ = rl.add_history_entry(&input);

                let pipeline = match parse(input.as_str()).unwrap() {
                    Some(p) => p,
                    None => continue,
                };
                let commands = pipeline.commands();

                // if commands.len() == 1 && commands[0].program() == "history" {
                //     let entries: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                //     let h = History::new(entries);
                //     h.execute(
                //         commands[0].args(),
                //         &mut std::io::stdout(),
                //         &mut std::io::stderr(),
                //     )
                //     .unwrap();
                //     continue;
                // }

                if commands.len() == 1 && commands[0].program() == "history" {
                    if let (Some(flag), Some(path)) =
                        (commands[0].args().first(), commands[0].args().get(1))
                    {
                        if flag == "-r" {
                            if let Ok(content) = std::fs::read_to_string(path) {
                                for line in content.lines() {
                                    if !line.is_empty() {
                                        let _ = rl.add_history_entry(line);
                                    }
                                }
                            }
                        } else if flag == "-a" {
                            let all_entries: Vec<String> =
                                rl.history().iter().map(|s| s.to_string()).collect();
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(path)
                            {
                                for entry in &all_entries[history_write_offset..] {
                                    let _ = writeln!(file, "{}", entry);
                                }
                                history_write_offset = all_entries.len();
                            }
                            // skip normal dispatch
                            let entries = all_entries;
                            builtins.insert(
                                "history".to_string(),
                                Box::new(History::new(entries, history_file_offset)),
                            );
                            continue;
                        }
                    }
                }

                let entries: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                builtins.insert(
                    "history".to_string(),
                    Box::new(History::new(entries, history_file_offset)),
                );

                let mut final_action = Action::Continue;
                let mut prev_stdin: Option<os_pipe::PipeReader> = None;
                let mut children: Vec<std::process::Child> = Vec::new();

                for (i, cmd) in commands.iter().enumerate() {
                    let stdin_override = prev_stdin.take();

                    let stdout_override = if i < commands.len() - 1 {
                        let (reader, writer) = os_pipe::pipe().unwrap();
                        prev_stdin = Some(reader);
                        Some(writer)
                    } else {
                        None
                    };

                    // let stdin_override = if i == 0 { None } else { prev_stdin.take() };
                    let (action, child) =
                        match execute_command(cmd, &builtins, stdin_override, stdout_override) {
                            Ok(result) => result,
                            Err(e) => {
                                eprintln!("{e}");
                                (Action::Continue, None)
                            }
                        };
                    if let Some(c) = child {
                        children.push(c);
                    }
                    final_action = action;
                    if final_action == Action::Exit {
                        break;
                    }
                }

                if final_action == Action::Exit {
                    // let _ = rl.append_history(&hist_file);
                    let entries: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                    let _ = std::fs::write(&hist_file, entries.join("\n") + "\n");

                    return Ok(());
                }

                for mut child in children {
                    child.wait().unwrap();
                }
            }
            Err(ReadlineError::Eof) => {
                let entries: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                let _ = std::fs::write(&hist_file, entries.join("\n") + "\n");

                break;
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(e) => {
                eprintln!("parse error: {e}");
                continue;
            }
        }
    }
    Ok(())
}
