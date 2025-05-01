use chrono::{Local, NaiveDate};

/// Holds all the parsed pieces of a todo line.
#[derive(Debug, Clone)]
pub struct Description {
    pub content: String,
    pub project: Option<String>,
    pub context: Option<String>,
    pub supplement: Option<String>,
    pub due: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct Todo {
    pub completion: bool,
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,
    pub description: Description,
}

impl Todo {
    /// Parse any todo.txt line into a `Todo`.
    pub fn parse(line: &str) -> Self {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        let mut idx = 0;

        let mut completion = false;
        let mut priority = None;
        let mut completion_date = None;
        let mut creation_date = None;

        // 1) done marker
        if tokens.get(idx) == Some(&"x") {
            completion = true;
            idx += 1;
        }

        // 2) (A) priority
        if let Some(&tok) = tokens.get(idx) {
            if tok.len() == 3 && tok.starts_with('(') && tok.ends_with(')') {
                priority = tok.chars().nth(1);
                idx += 1;
            }
        }

        // 3) up to two dates
        let d1 = tokens.get(idx).and_then(|t| NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());
        let d2 = tokens.get(idx + 1).and_then(|t| NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());
        match (completion, d1, d2) {
            (true, Some(cd), Some(cr)) => {
                completion_date = Some(cd);
                creation_date = Some(cr);
                idx += 2;
            }
            (false, Some(cr), _) => {
                creation_date = Some(cr);
                idx += 1;
            }
            _ => {}
        }

        // 4) description + tags + due
        let mut content = Vec::new();
        let mut project = None;
        let mut context = None;
        let mut supplement = None;
        let mut due = None;

        for &word in &tokens[idx..] {
            if word.starts_with('+') {
                project = Some(word[1..].to_string());
            } else if word.starts_with('@') {
                context = Some(word[1..].to_string());
            } else if word.starts_with("due:") {
                if let Ok(d) = NaiveDate::parse_from_str(&word[4..], "%Y-%m-%d") {
                    due = Some(d);
                }
                supplement = Some(word.to_string());
            } else if supplement.is_none() && (word.contains(':') || word.contains('=')) {
                supplement = Some(word.to_string());
            } else {
                content.push(word.to_string());
            }
        }

        Self {
            completion,
            priority,
            completion_date,
            creation_date,
            description: Description {
                content: content.join(" "),
                project,
                context,
                supplement,
                due,
            },
        }
    }

    /// Serialize back to a todo.txt–compatible line.
    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        if self.completion {
            parts.push("x".into());
        }
        if let Some(p) = self.priority {
            parts.push(format!("({})", p));
        }
        if self.completion {
            if let Some(cd) = self.completion_date {
                parts.push(cd.format("%Y-%m-%d").to_string());
            }
        }
        if let Some(cr) = self.creation_date {
            parts.push(cr.format("%Y-%m-%d").to_string());
        }
        parts.push(self.description.content.clone());
        if let Some(proj) = &self.description.project {
            parts.push(format!("+{}", proj));
        }
        if let Some(ctx) = &self.description.context {
            parts.push(format!("@{}", ctx));
        }
        if let Some(sup) = &self.description.supplement {
            parts.push(sup.clone());
        }
        parts.join(" ")
    }

    /// Mark done today.
    pub fn mark_done(&mut self) {
        if !self.completion {
            self.completion = true;
            self.completion_date = Some(Local::now().date_naive());
        }
    }

    /// For `add`: enforce non-empty content and stamp creation date.
    pub fn from_add(input: &str) -> Result<Self, String> {
        let mut todo = Self::parse(input);
        if todo.description.content.trim().is_empty() {
            return Err("Task must include non-empty description".into());
        }
        if todo.creation_date.is_none() {
            todo.creation_date = Some(Local::now().date_naive());
        }
        Ok(todo)
    }
}
