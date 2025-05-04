use crate::todo::Todo;
use std::fs::{read_to_string, OpenOptions};
use std::io::{self, Write};

/// `todo.txt` から全件ロード
pub fn load_all() -> Vec<Todo> {
    match read_to_string("todo.txt") {
        Ok(txt) => txt.lines().map(Todo::parse).collect(),
        Err(_) => Vec::new(),
    }
}

/// `todo.txt` を上書き
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

/// `todo.txt` に追記
pub fn append_one(todo: &Todo) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("todo.txt")?;
    writeln!(f, "{}", todo.format())?;
    Ok(())
}