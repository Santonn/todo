use chrono::{Local, NaiveDate};
use ratatui::style::Color;

/// 説明部分
#[derive(Debug, Clone)]
pub struct Description {
    pub content: String,
    pub project: Option<String>,
    pub context: Option<String>,
    pub supplement: Option<String>,
    pub due: Option<NaiveDate>,
}

/// Todo 本体
#[derive(Debug, Clone)]
pub struct Todo {
    pub completion: bool,
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,
    pub description: Description,
}

impl Todo {
    /// パース
    pub fn parse(line: &str) -> Self {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        let mut idx = 0;
        let mut completion = false;
        let mut priority = None;
        let mut completion_date = None;
        let mut creation_date = None;

        // 完了マーカー
        if tokens.get(idx) == Some(&"x") {
            completion = true;
            idx += 1;
        }
        // 優先度
        if let Some(&tok) = tokens.get(idx) {
            if tok.len() == 3 && tok.starts_with('(') && tok.ends_with(')') {
                priority = tok.chars().nth(1);
                idx += 1;
            }
        }
        // 日付 (完了 or 作成)
        let d1 = tokens.get(idx).and_then(|t| NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());
        let d2 = tokens.get(idx+1).and_then(|t| NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());
        match (completion, d1, d2) {
            (true, Some(cd), Some(cr)) => { completion_date = Some(cd); creation_date = Some(cr); idx += 2; }
            (false, Some(cr), _)       => { creation_date = Some(cr); idx += 1; }
            _ => {}
        }
        // 内容 + タグ + due
        let mut content = Vec::new();
        let mut project = None;
        let mut context = None;
        let mut supplement = None;
        let mut due = None;
        for &w in &tokens[idx..] {
            if w.starts_with('+') {
                project = Some(w[1..].to_string());
            } else if w.starts_with('@') {
                context = Some(w[1..].to_string());
            } else if w.starts_with("due:") {
                if let Ok(d) = NaiveDate::parse_from_str(&w[4..], "%Y-%m-%d") {
                    due = Some(d);
                }
                supplement = Some(w.to_string());
            } else if supplement.is_none() && (w.contains(':') || w.contains('=')) {
                supplement = Some(w.to_string());
            } else {
                content.push(w.to_string());
            }
        }
        Self {
            completion,
            priority,
            completion_date,
            creation_date,
            description: Description { content: content.join(" "), project, context, supplement, due },
        }
    }

    /// シリアライズ
    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        if self.completion { parts.push("x".into()); }
        if let Some(p) = self.priority { parts.push(format!("({})", p)); }
        if self.completion {
            if let Some(cd) = self.completion_date { parts.push(cd.format("%Y-%m-%d").to_string()); }
        }
        if let Some(cr) = self.creation_date { parts.push(cr.format("%Y-%m-%d").to_string()); }
        parts.push(self.description.content.clone());
        if let Some(proj) = &self.description.project { parts.push(format!("+{}", proj)); }
        if let Some(ctx) = &self.description.context { parts.push(format!("@{}", ctx)); }
        if let Some(sup) = &self.description.supplement { parts.push(sup.clone()); }
        parts.join(" ")
    }

    /// 完了マーク (今日の日付)
    pub fn mark_done(&mut self) {
        if !self.completion {
            self.completion = true;
            self.completion_date = Some(Local::now().date_naive());
        }
    }

    /// `add` 用パーサ
    pub fn from_add(input: &str) -> Result<Self, String> {
        let mut t = Self::parse(input);
        if t.description.content.trim().is_empty() {
            return Err("Task must include non-empty description".into());
        }
        if t.creation_date.is_none() {
            t.creation_date = Some(Local::now().date_naive());
        }
        Ok(t)
    }

    /// 未完了タスクの due 日取得
    pub fn due_uncompleted(&self) -> Option<NaiveDate> {
        if !self.completion { self.description.due } else { None }
    }

    // 未完了タスクの priority 取得
    pub fn priority_uncompleted(&self) -> Option<char> {
        if !self.completion { self.priority } else { None }
    }

    /// マーカー色判定
    pub fn marker_color(&self, today: NaiveDate) -> Color {
        use ratatui::style::Color;
        if let Some(due) = self.description.due {
            let days = (due - today).num_days();
            if days <= 3 { Color::Red }
            else if days <= 7 { Color::Yellow }
            else { Color::Green }
        } else {
            Color::Gray
        }
    }
}