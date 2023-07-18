use ambient_api::{components::core, ecs::GeneralQuery, prelude::*};
use rhai::plugin::*;
use std::sync::{Arc, Mutex};

pub struct Console {
    engine: rhai::Engine,
    lines: Vec<ConsoleLine>,
    on_output: Option<Box<dyn FnMut() + Send + Sync>>,
    is_server: bool,
}
impl std::fmt::Debug for Console {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Console")
            .field("lines", &self.lines)
            .finish()
    }
}
impl Console {
    pub fn new(is_server: bool) -> Arc<Mutex<Self>> {
        let mut console = Self {
            engine: rhai::Engine::new(),
            lines: Vec::new(),
            on_output: None,
            is_server,
        };

        let ctx = ConsoleContext::new();

        {
            let engine = console.engine();
            engine.set_default_tag(Dynamic::from(ctx));

            let wasm_module = exported_module!(wasm);
            engine.register_static_module("wasm", wasm_module.into());
        }

        Arc::new(Mutex::new(console))
    }

    pub fn engine(&mut self) -> &mut rhai::Engine {
        &mut self.engine
    }

    pub fn lines(&self) -> &[ConsoleLine] {
        self.lines.as_ref()
    }

    pub fn on_output<F: FnMut() + Send + Sync + 'static>(&mut self, on_output: F) {
        self.on_output = Some(Box::new(on_output));
    }

    pub fn clear_on_output(&mut self) {
        self.on_output = None;
    }

    pub fn input(
        &mut self,
        text: &str,
        mut output: impl FnMut(ConsoleLine) + Send + Sync + 'static,
    ) {
        self.input_impl(text, &mut output);
    }

    pub fn push(
        &mut self,
        line: ConsoleLine,
        output: Option<&mut (dyn FnMut(ConsoleLine) + Send + Sync + 'static)>,
    ) {
        self.lines.push(line.clone());
        if self.lines.len() > 100 {
            self.lines.drain(0..(self.lines.len() - 100));
        }
        if let Some(output) = output {
            output(line);
        }
        if let Some(on_update) = &mut self.on_output {
            on_update();
        }
    }
}
impl Console {
    fn input_impl(
        &mut self,
        text: &str,
        output: &mut (dyn FnMut(ConsoleLine) + Send + Sync + 'static),
    ) {
        self.push(
            ConsoleLine {
                text: format!("> {}", text),
                ty: ConsoleLineType::User,
                is_server: self.is_server,
            },
            Some(output),
        );
        match self.engine.eval::<rhai::Dynamic>(text) {
            Ok(result) => {
                if result.is_unit() {
                    return;
                }
                self.push(
                    ConsoleLine {
                        text: format!("= {}", result),
                        ty: ConsoleLineType::Normal,
                        is_server: self.is_server,
                    },
                    Some(output),
                );
            }
            Err(error) => {
                self.push(
                    ConsoleLine {
                        text: format!("{}", error),
                        ty: ConsoleLineType::Error,
                        is_server: self.is_server,
                    },
                    Some(output),
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleLineType {
    Normal,
    User,
    Error,
}
impl TryFrom<u8> for ConsoleLineType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::User),
            2 => Ok(Self::Error),
            _ => Err(()),
        }
    }
}
impl Into<u8> for ConsoleLineType {
    fn into(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::User => 1,
            Self::Error => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub text: String,
    pub ty: ConsoleLineType,
    pub is_server: bool,
}

#[derive(Clone)]
struct ConsoleContext {
    module_query: GeneralQuery<(Component<String>, Component<bool>)>,
}
impl ConsoleContext {
    fn new() -> Self {
        ConsoleContext {
            module_query: query((core::wasm::module_name(), core::wasm::module_enabled())).build(),
        }
    }
}

#[export_module]
mod wasm {
    #[cfg(feature = "client")]
    pub fn list(ctx: NativeCallContext) -> Dynamic {
        list_internal(ctx, ListFilter::Client)
    }

    #[cfg(feature = "server")]
    pub fn list_client(ctx: NativeCallContext) -> Dynamic {
        list_internal(ctx, ListFilter::Client)
    }

    #[cfg(feature = "server")]
    pub fn list_server(ctx: NativeCallContext) -> Dynamic {
        list_internal(ctx, ListFilter::Server)
    }

    enum ListFilter {
        Client,
        Server,
    }
    fn list_internal(ctx: NativeCallContext, filter: ListFilter) -> Dynamic {
        let ctx = ctx.tag().unwrap().clone_cast::<ConsoleContext>();
        let modules = ctx.module_query.evaluate();

        modules
            .into_iter()
            .filter(|(id, _)| {
                let on_server = entity::has_component(*id, core::wasm::module_on_server());
                match filter {
                    ListFilter::Client => !on_server,
                    ListFilter::Server => on_server,
                }
            })
            .map(|(_, (name, enabled))| format!("{name}: {enabled}"))
            .collect::<Vec<_>>()
            .into()
    }
}
