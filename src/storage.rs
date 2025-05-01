use crate::todo::Todo;
use std::fs::{read_to_string, OpenOptions};
use std::io::{self, Write};

/// Load *all* todos from `todo.txt`, one per line.
pub fn load_all() -> Vec<Todo> {
    match read_to_string("todo.txt") {
        Ok(txt) => txt.lines().map(Todo::parse).collect(),
        Err(_) => Vec::new(),
    }
}

/// Overwrite `todo.txt` with the given list of todos.
pub fn rewrite_file(todos: &[Todo]) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("todo.txt")?;
    for t in todos {
        writeln!(f, "{}", t.format())?;
    }
    Ok(())
}

/// Append one todo to `todo.txt`.
pub fn append_one(todo: &Todo) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("todo.txt")?;
    writeln!(f, "{}", todo.format())?;
    Ok(())
}
