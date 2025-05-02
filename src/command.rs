use crate::storage::{append_one, load_all, rewrite_file};
use crate::todo::Todo;

/// Result of executing a command — returns updated todo list, current view, and any error message.
pub struct CommandResult {
    pub todos: Vec<Todo>,
    pub view: Vec<usize>,
    pub error: Option<String>,
}

/// Executes a command string and returns updated state.
pub fn execute_command(
    todos: Vec<Todo>,
    view: Vec<usize>,
    input: &str,
) -> CommandResult {
    let mut todos = todos;
    let mut view = view;
    let mut error = None;
    let cmd = input.trim();

    match cmd.split_whitespace().next() {
        Some("list") => {
            todos = load_all();
            view = (0..todos.len()).collect();
        }
        Some("add") => {
            if let Some(rest) = cmd.strip_prefix("add ").map(str::trim) {
                match Todo::from_add(rest) {
                    Ok(t) => {
                        if append_one(&t).is_ok() {
                            todos.push(t);
                            view = (0..todos.len()).collect();
                        }
                    }
                    Err(e) => error = Some(e),
                }
            } else {
                error = Some("Usage: add <todo>".into());
            }
        }
        Some("done") => {
            if let Some(arg) = cmd.strip_prefix("done ").map(str::trim) {
                if let Ok(id) = arg.parse::<usize>() {
                    if id >= 1 && id <= view.len() {
                        let idx = view[id - 1];
                        todos[idx].mark_done();
                        let _ = rewrite_file(&todos);
                        todos = load_all();
                        view = (0..todos.len()).collect();
                    } else {
                        error = Some("Invalid ID".into());
                    }
                } else {
                    error = Some("Usage: done <ID>".into());
                }
            }
        }
        Some("remove") => {
            if let Some(arg) = cmd.strip_prefix("remove ").map(str::trim) {
                if let Ok(id) = arg.parse::<usize>() {
                    if id >= 1 && id <= view.len() {
                        let idx = view[id - 1];
                        todos.remove(idx);
                        let _ = rewrite_file(&todos);
                        todos = load_all();
                        view = (0..todos.len()).collect();
                    } else {
                        error = Some("Invalid ID".into());
                    }
                } else {
                    error = Some("Usage: remove <ID>".into());
                }
            }
        }
        Some("closest") => {
            // Filter and sort by nearest due date
            let mut pairs: Vec<(usize, chrono::NaiveDate)> = todos
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    if !t.completion {
                        t.description.due.map(|d| (i, d))
                    } else {
                        None
                    }
                })
                .collect();
            pairs.sort_by_key(|&(_, d)| d);
            view = pairs.into_iter().map(|(i, _)| i).collect();
        }
        Some("important") => {
            let mut filtered: Vec<(usize, char)> = todos
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    if !t.completion {
                        t.priority.map(|p| (i, p))
                    } else {
                        None
                    }
                })
                .collect();
        
            filtered.sort_by_key(|&(_, p)| p);
            view = filtered.into_iter().map(|(i, _)| i).collect();
        }
        
        
        
        
        Some("") | None => {}
        Some(other) => {
            error = Some(format!("Unknown command: {}", other));
        }
    }

    CommandResult { todos, view, error }
}
