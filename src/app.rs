use crate::command::execute_command;
use crate::todo::Todo;
use crate::storage::load_all;
use chrono::Local;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum InputMode { Normal, Editing, Focused }

#[derive(Clone, Copy)]
enum Focus { Input, Due, NoDue }

pub struct App {
    todos: Vec<Todo>,
    view: Vec<usize>,
    input: String,
    cursor: usize,
    mode: InputMode,
    error: Option<String>,

    focus: Focus,
    due_scroll: usize,
    nodue_scroll: usize,
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
            focus: Focus::Input,
            due_scroll: 0,
            nodue_scroll: 0,
        }
    }

    fn apply_command(&mut self) {
        let res = execute_command(&mut self.todos, &mut self.view, &self.input);
        self.error = res.error;
        self.input.clear();
        self.cursor = 0;
    }

    fn cursor_x(&self) -> u16 {
        let end = self.input.char_indices().nth(self.cursor).map(|(i, _)| i).unwrap_or(self.input.len());
        UnicodeWidthStr::width(&self.input[..end]) as u16
    }

    pub fn run(mut self, mut term: DefaultTerminal) -> Result<()> {
        loop {
            term.draw(|f| self.draw(f))?;
            if let Event::Key(key) = event::read()? {
                let current_mode = self.mode;
                match (current_mode, key.kind) {
                    (InputMode::Normal, KeyEventKind::Press) => match key.code {
                        KeyCode::Char('e') => {
                            self.mode = match self.focus {
                                Focus::Input => InputMode::Editing,
                                _ => InputMode::Focused,
                            };
                        },
                        KeyCode::Char('q') => break,
                        KeyCode::Up => match self.focus {
                            Focus::Due | Focus::NoDue => self.focus = Focus::Input,
                            _ => {}
                        },
                        KeyCode::Down => match self.focus {
                            Focus::Input => self.focus = Focus::Due,
                            Focus::Due => self.focus = Focus::NoDue,
                            _ => {}
                        },
                        KeyCode::Left => if let Focus::NoDue = self.focus {
                            self.focus = Focus::Due;
                        },
                        KeyCode::Right => if let Focus::Due = self.focus {
                            self.focus = Focus::NoDue;
                        },
                        _ => {}
                    },
                    (InputMode::Editing, KeyEventKind::Press) => match key.code {
                        KeyCode::Enter => self.apply_command(),
                        KeyCode::Char(c) => {
                            let idx = self.input.char_indices().map(|(i, _)| i)
                                .nth(self.cursor).unwrap_or(self.input.len());
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
                        KeyCode::Right => self.cursor = (self.cursor + 1).min(self.input.chars().count()),
                        KeyCode::Esc => self.mode = InputMode::Normal,
                        _ => {}
                    },
                    (InputMode::Focused, KeyEventKind::Press) => match key.code {
                        KeyCode::Up => match self.focus {
                            Focus::Due => self.due_scroll = self.due_scroll.saturating_sub(1),
                            Focus::NoDue => self.nodue_scroll = self.nodue_scroll.saturating_sub(1),
                            _ => {}
                        },
                        KeyCode::Down => match self.focus {
                            Focus::Due => self.due_scroll += 1,
                            Focus::NoDue => self.nodue_scroll += 1,
                            _ => {}
                        },
                        KeyCode::Esc => self.mode = InputMode::Normal,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        let today = Local::now().date_naive();
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]).split(f.area());

        let header = if let Some(err) = &self.error {
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Red))
        } else {
            let (msg, style) = match self.mode {
                InputMode::Normal => (
                    vec!["Press ".into(), "e".bold(), " to focus block, arrows to move focus, q to quit.".into()],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec!["Press ".into(), "Esc".bold(), " to cancel, ".into(), "Enter".bold(), " to run.".into()],
                    Style::default(),
                ),
                InputMode::Focused => (
                    vec!["Use arrows to scroll list, ".into(), "Esc".bold(), " to return.".into()],
                    Style::default(),
                ),
            };
            Paragraph::new(Text::from(Line::from(msg)).patch_style(style))
        };
        f.render_widget(header, chunks[0]);

        let input_block = if let Focus::Input = self.focus {
            Block::bordered().title("Input").border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::bordered().title("Input")
        };
        let input = Paragraph::new(self.input.as_str())
            .style(if matches!(self.mode, InputMode::Editing) {
                Style::default().fg(Color::Yellow)
            } else { Style::default() })
            .block(input_block);
        f.render_widget(input, chunks[1]);
        if matches!(self.mode, InputMode::Editing) {
            f.set_cursor_position((chunks[1].x + self.cursor_x() + 1, chunks[1].y + 1));
        }

        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[2]);
        let sep = |w: u16| Line::from(vec![Span::raw(" "), Span::raw("-".repeat((w - 2) as usize))]);

        let mut due_items = Vec::new();
        let mut nodue_items = Vec::new();

        for (idx, &i) in self.view.iter().enumerate() {
            let t = &self.todos[i];
            let marker_color = t.marker_color(today);
            let marker = Span::styled(" ", Style::default().bg(marker_color));
            let mut lines = Vec::new();
            lines.push(sep(cols[0].width));
            let head = format!("{}: {}{}{}",
                idx + 1,
                if t.completion { "x " } else { "" },
                t.priority.map(|p| format!("({}) ", p)).unwrap_or_default(),
                t.description.content);
            lines.push(Line::from(vec![marker.clone(), Span::raw(head)]));
            if t.completion_date.is_some() || t.creation_date.is_some() {
                let cd = t.completion_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                let cr = t.creation_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                lines.push(Line::from(vec![marker.clone(), Span::raw(format!("     {} {}", cd, cr))]));
            }
            if let Some(p) = &t.description.project {
                lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      +{}", p))]));
            }
            if let Some(c) = &t.description.context {
                lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      @{}", c))]));
            }
            if let Some(d) = t.description.due {
                lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      due:{}", d.format("%Y-%m-%d"))) ]));
            }
            lines.push(sep(cols[0].width));

            let item = ListItem::new(Text::from(lines));
            if t.description.due.is_some() {
                due_items.push(item);
            } else {
                nodue_items.push(item);
            }
        }

        let due_block = if let Focus::Due = self.focus {
            Block::bordered().title("Due Todos").border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::bordered().title("Due Todos")
        };
        let due_end = (self.due_scroll + 10).min(due_items.len());
        let visible_due = due_items.get(self.due_scroll..due_end).unwrap_or(&[]);
        f.render_widget(List::new(visible_due.to_vec()).block(due_block), cols[0]);

        let nodue_block = if let Focus::NoDue = self.focus {
            Block::bordered().title("No‑Due Todos").border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::bordered().title("No‑Due Todos")
        };
        let nodue_end = (self.nodue_scroll + 10).min(nodue_items.len());
        let visible_nodue = nodue_items.get(self.nodue_scroll..nodue_end).unwrap_or(&[]);
        f.render_widget(List::new(visible_nodue.to_vec()).block(nodue_block), cols[1]);
    }
}
