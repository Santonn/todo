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

enum InputMode { Normal, Editing }

pub struct App {
    todos: Vec<Todo>,
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
        Self { todos, view, input: String::new(), cursor: 0, mode: InputMode::Normal, error: None }
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
                match self.mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.mode = InputMode::Editing,
                        KeyCode::Char('q') => break,
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
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

        // ヘッダー
        let header = if let Some(err) = &self.error {
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Red))
        } else {
            let (msg, style) = match self.mode {
                InputMode::Normal => (
                    vec!["Press ".into(), "q".bold(), " to quit, ".into(), "e".bold(), " to edit.".into()],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec!["Press ".into(), "Esc".bold(), " to cancel, ".into(), "Enter".bold(), " to run.".into()],
                    Style::default(),
                ),
            };
            Paragraph::new(Text::from(Line::from(msg)).patch_style(style))
        };
        f.render_widget(header, chunks[0]);

        // 入力欄
        let input = Paragraph::new(self.input.as_str())
            .style(if matches!(self.mode, InputMode::Editing) { Style::default().fg(Color::Yellow) } else { Style::default() })
            .block(Block::bordered().title("Input"));
        f.render_widget(input, chunks[1]);
        if matches!(self.mode, InputMode::Editing) {
            f.set_cursor_position((chunks[1].x + self.cursor_x() + 1, chunks[1].y + 1));
        }

        // TODO リスト表示
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[2]);
        let sep = |w: u16| Line::from(vec![Span::raw(" "), Span::raw("-".repeat((w - 2) as usize))]);

        let mut due_items = Vec::new();
        let mut nodue_items = Vec::new();

        for (idx, &i) in self.view.iter().enumerate() {
            let t = &self.todos[i];
            // マーカー色
            let marker_color = t.marker_color(today);
            let marker = Span::styled(" ", Style::default().bg(marker_color));

            let mut lines = Vec::new();
            lines.push(sep(cols[0].width));
            // 見出し行
            let head = format!("{}: {}{}{}",
                idx + 1,
                if t.completion { "x " } else { "" },
                t.priority.map(|p| format!("({}) ", p)).unwrap_or_default(),
                t.description.content
            );
            lines.push(Line::from(vec![marker.clone(), Span::raw(head)]));
            // 日付行
            if t.completion_date.is_some() || t.creation_date.is_some() {
                let cd = t.completion_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                let cr = t.creation_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                lines.push(Line::from(vec![marker.clone(), Span::raw(format!("     {} {}", cd, cr))]));
            }
            // タグ行 & due
            if let Some(p) = &t.description.project { lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      +{}", p))])); }
            if let Some(c) = &t.description.context { lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      @{}", c))])); }
            if let Some(d) = t.description.due { lines.push(Line::from(vec![marker.clone(), Span::raw(format!("      due:{}", d.format("%Y-%m-%d"))) ])); }
            lines.push(sep(cols[0].width));

            let item = ListItem::new(Text::from(lines));
            if t.description.due.is_some() {
                due_items.push(item);
            } else {
                nodue_items.push(item);
            }
        }

        f.render_widget(List::new(due_items).block(Block::bordered().title("Due Todos")), cols[0]);
        f.render_widget(List::new(nodue_items).block(Block::bordered().title("No-Due Todos")), cols[1]);
    }
}