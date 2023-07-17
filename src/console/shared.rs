use ambient_api::prelude::*;

pub struct Console {
    lines: Vec<ConsoleLine>,
    on_update: Option<Box<dyn FnMut() + Send + Sync>>,
}
impl std::fmt::Debug for Console {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Console")
            .field("lines", &self.lines)
            .finish()
    }
}
impl Console {
    pub fn new() -> Self {
        Self {
            lines: (0..10)
                .map(|i| ConsoleLine {
                    text: format!("Line {i}"),
                    ty: ConsoleLineType::Normal,
                })
                .collect::<Vec<_>>(),
            on_update: None,
        }
    }

    pub fn lines(&self) -> &[ConsoleLine] {
        self.lines.as_ref()
    }

    pub fn on_update<F: FnMut() + Send + Sync + 'static>(&mut self, on_update: F) {
        self.on_update = Some(Box::new(on_update));
    }

    pub fn clear_update(&mut self) {
        self.on_update = None;
    }

    pub fn input(&mut self, text: &str) {
        self.push(ConsoleLine {
            text: format!("> {}", text),
            ty: ConsoleLineType::User,
        });
    }
}
impl Console {
    fn push(&mut self, line: ConsoleLine) {
        self.lines.push(line);
        if self.lines.len() > 100 {
            self.lines.drain(0..(self.lines.len() - 100));
        }
        if let Some(on_update) = &mut self.on_update {
            on_update();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleLineType {
    Normal,
    User,
}
impl From<ConsoleLineType> for Vec4 {
    fn from(value: ConsoleLineType) -> Self {
        match value {
            ConsoleLineType::Normal => vec4(0.8, 0.8, 0.8, 1.0),
            ConsoleLineType::User => vec4(0.0, 0.8, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub text: String,
    pub ty: ConsoleLineType,
}
