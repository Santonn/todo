
use crate::storage::{append_one, load_all, rewrite_file};
use crate::todo::Todo;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    /// All todos, in file order
    todos: Vec<Todo>,
    /// Which indices of `todos` are currently shown, in what order
    view: Vec<usize>,
    input: String,
    cursor: usize,
    mode: InputMode,
    error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let todos = load_all();
        let view = (0..todos.len()).collect();
        Self {
            todos,
            view,
            input: String::new(),
            cursor: 0,
            mode: InputMode::Normal,
            error: None,
        }
    }

    /// Refresh both `todos` and full `view`
    fn refresh_all(&mut self) {
        self.todos = load_all();
        self.view = (0..self.todos.len()).collect();
    }

    fn execute_command(&mut self) {
        let cmd_owned = self.input.clone();
        let cmd = cmd_owned.trim();
        self.error = None;

        match cmd.split_whitespace().next() {
            Some("list") => {
                self.refresh_all();
            }
            Some("add") => {
                if let Some(rest) = cmd.strip_prefix("add ").map(str::trim) {
                    match Todo::from_add(rest) {
                        Ok(t) => {
                            if append_one(&t).is_ok() {
                                self.todos.push(t);
                                // new item appears at end of full list
                                self.view.push(self.todos.len() - 1);
                            }
                        }
                        Err(e) => self.error = Some(e),
                    }
                } else {
                    self.error = Some("Usage: add <todo>".into());
                }
            }
            Some("done") => {
                // operate on current view
                if let Some(arg) = cmd.strip_prefix("done ").map(str::trim) {
                    if let Ok(id) = arg.parse::<usize>() {
                        if id >= 1 && id <= self.view.len() {
                            let todo_idx = self.view[id - 1];
                            self.todos[todo_idx].mark_done();
                            let _ = rewrite_file(&self.todos);
                            self.refresh_all();
                        } else {
                            self.error = Some("Invalid ID".into());
                        }
                    } else {
                        self.error = Some("Usage: done <ID>".into());
                    }
                }
            }
            Some("remove") => {
                if let Some(arg) = cmd.strip_prefix("remove ").map(str::trim) {
                    if let Ok(id) = arg.parse::<usize>() {
                        if id >= 1 && id <= self.view.len() {
                            let todo_idx = self.view[id - 1];
                            self.todos.remove(todo_idx);
                            let _ = rewrite_file(&self.todos);
                            self.refresh_all();
                        } else {
                            self.error = Some("Invalid ID".into());
                        }
                    } else {
                        self.error = Some("Usage: remove <ID>".into());
                    }
                }
            }
            Some("closest") => {
                // build a new view: only incomplete with due, sorted
                let mut pairs: Vec<(usize, Todo)> = self
                    .todos
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| !t.completion && t.description.due.is_some())
                    .map(|(i, t)| (i, t.clone()))
                    .collect();
                pairs.sort_by_key(|(_, t)| t.description.due.unwrap());
                self.view = pairs.into_iter().map(|(i, _)| i).collect();
            }
            Some("") | None => { /* no-op */ }
            Some(other) => {
                self.error = Some(format!("Unknown command: {}", other));
            }
        }

        self.input.clear();
        self.cursor = 0;
    }

    fn cursor_x(&self) -> u16 {
        let s: String = self.input.chars().take(self.cursor).collect();
        UnicodeWidthStr::width(s.as_str()) as u16
    }

    pub fn run(mut self, mut term: DefaultTerminal) -> Result<()> {
        loop {
            term.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                match self.mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.mode = InputMode::Editing,
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.execute_command(),
                        KeyCode::Char(c) => {
                            let idx = self
                                .input
                                .char_indices()
                                .map(|(i, _)| i)
                                .nth(self.cursor)
                                .unwrap_or(self.input.len());
                            self.input.insert(idx, c);
                            self.cursor += 1;
                        }
                        KeyCode::Backspace => {
                            if self.cursor > 0 {
                                let mut cs: Vec<char> = self.input.chars().collect();
                                cs.remove(self.cursor - 1);
                                self.input = cs.into_iter().collect();
                                self.cursor -= 1;
                            }
                        }
                        KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
                        KeyCode::Right => {
                            self.cursor = (self.cursor + 1).min(self.input.chars().count())
                        }
                        KeyCode::Esc => self.mode = InputMode::Normal,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn draw(&self, f: &mut Frame) {
        let chunks: [Rect; 3] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .areas(f.area());

        // top line: error or help
        let header = if let Some(err) = &self.error {
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Red))
        } else {
            let (msg, style) = match self.mode {
                InputMode::Normal => (
                    vec![
                        "Press ".into(),
                        "q".bold(),
                        " to quit, ".into(),
                        "e".bold(),
                        " to edit.".into(),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec![
                        "Press ".into(),
                        "Esc".bold(),
                        " to cancel, ".into(),
                        "Enter".bold(),
                        " to run.".into(),
                    ],
                    Style::default(),
                ),
            };
            Paragraph::new(Text::from(Line::from(msg)).patch_style(style))
        };
        f.render_widget(header, chunks[0]);

        // input
        let input_w = Paragraph::new(self.input.as_str())
            .style(match self.mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        f.render_widget(input_w, chunks[1]);
        if let InputMode::Editing = self.mode {
            f.set_cursor_position(Position::new(
                chunks[1].x + self.cursor_x() + 1,
                chunks[1].y + 1,
            ));
        }

        // todos list drawn from `view`
        let items: Vec<ListItem> = self
            .view
            .iter()
            .enumerate()
            .map(|(display_idx, &todo_idx)| {
                let t = &self.todos[todo_idx];
                let mut line = format!("{}: {}", display_idx + 1, t.format());
                if let Some(due) = &t.description.due {
                    line.push_str(&format!(" [due {}]", due));
                }
                ListItem::new(Line::from(Span::raw(line)))
            })
            .collect();

        let list_w = List::new(items).block(Block::bordered().title("Todos"));
        f.render_widget(list_w, chunks[2]);
    }
}
