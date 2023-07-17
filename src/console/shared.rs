use ambient_api::prelude::*;
use std::sync::{Arc, Mutex};

pub struct Console {
    engine: rhai::Engine,
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
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            engine: rhai::Engine::new(),
            lines: (0..10)
                .map(|i| ConsoleLine {
                    text: format!("Line {i}"),
                    ty: ConsoleLineType::Normal,
                })
                .collect::<Vec<_>>(),
            on_update: None,
        }))
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
        match self.engine.eval::<rhai::Dynamic>(text) {
            Ok(result) => {
                self.push(ConsoleLine {
                    text: format!("= {}", result),
                    ty: ConsoleLineType::Normal,
                });
            }
            Err(error) => {
                self.push(ConsoleLine {
                    text: format!("{}", error),
                    ty: ConsoleLineType::Error,
                });
            }
        }
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
    Error,
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub text: String,
    pub ty: ConsoleLineType,
}
