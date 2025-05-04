use crate::storage::{append_one, load_all, rewrite_file};
use crate::todo::Todo;
use chrono::NaiveDate;

/// コマンドの種類
pub enum Command {
    List,
    Add(String),
    Done(usize),
    Remove(usize),
    Closest,
    Important,
    Empty,
    Unknown(String),
}

impl Command {
    pub fn parse(input: &str) -> Self {
        let cmd = input.trim();
        if cmd.is_empty() {
            return Command::Empty;
        }
        let mut parts = cmd.splitn(2, ' ');
        match parts.next().unwrap() {
            "list"      => Command::List,
            "add"       => parts.next().map(|s| Command::Add(s.to_string())).unwrap_or(Command::Unknown(cmd.into())),
            "done"      => parts.next()
                                .and_then(|s| s.parse().ok())
                                .map(Command::Done)
                                .unwrap_or(Command::Unknown(cmd.into())),
            "rm"    => parts.next()
                                .and_then(|s| s.parse().ok())
                                .map(Command::Remove)
                                .unwrap_or(Command::Unknown(cmd.into())),
            "sd"   => Command::Closest,
            "sp" => Command::Important,
            other        => Command::Unknown(other.into()),
        }
    }
}

/// コマンド実行結果
pub struct CommandResult {
    pub error: Option<String>,
}

/// コマンドを実行して `todos` / `view` を更新
pub fn execute_command(
    todos: &mut Vec<Todo>,
    view: &mut Vec<usize>,
    input: &str,
) -> CommandResult {
    let cmd = Command::parse(input);
    let mut error = None;

    match cmd {
        Command::Empty => {}
        Command::List => {
            *todos = load_all();
            *view = (0..todos.len()).collect();
        }
        Command::Add(text) => match Todo::from_add(&text) {
            Ok(t) if append_one(&t).is_ok() => {
                todos.push(t);
                *view = (0..todos.len()).collect();
            }
            Ok(_) => error = Some("Failed to append todo".into()),
            Err(e) => error = Some(e),
        },
        Command::Done(id) => {
            if let Some(&idx) = view.get(id.saturating_sub(1)) {
                todos[idx].mark_done();
                let _ = rewrite_file(todos);
                *todos = load_all();
                *view = (0..todos.len()).collect();
            } else {
                error = Some("Invalid ID".into());
            }
        }
        Command::Remove(id) => {
            if let Some(&idx) = view.get(id.saturating_sub(1)) {
                todos.remove(idx);
                let _ = rewrite_file(todos);
                *todos = load_all();
                *view = (0..todos.len()).collect();
            } else {
                error = Some("Invalid ID".into());
            }
        }
        Command::Closest => {
            let mut pairs: Vec<(usize, NaiveDate)> = todos
                .iter()
                .enumerate()
                .filter_map(|(i, t)| t.due_uncompleted().map(|d| (i, d)))
                .collect();
            pairs.sort_by_key(|&(_, d)| d);
            *view = pairs.into_iter().map(|(i, _)| i).collect();
        }
        Command::Important => {
            let mut pairs: Vec<(usize, char)> = todos
                .iter()
                .enumerate()
                .filter_map(|(i, t)| t.priority_uncompleted().map(|p| (i, p)))
                .collect();
            pairs.sort_by_key(|&(_, p)| p);
            *view = pairs.into_iter().map(|(i, _)| i).collect();
        }
        Command::Unknown(s) => error = Some(format!("Unknown command: {}", s)),
    }

    CommandResult { error }
}