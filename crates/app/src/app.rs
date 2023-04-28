use crate::{window::AmbientWindow, AmbientWindowBuilder};
use ambient_core::window::window_scale_factor;
use ambient_ecs::World;
use ambient_sys::task::RuntimeHandle;
use std::{collections::HashMap, future::Future};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

pub struct AppBuilder {
    main_window: Option<AmbientWindowBuilder>,
    event_loop: Option<EventLoop<()>>,
}
impl AppBuilder {
    pub fn new(main_window: AmbientWindowBuilder) -> Self {
        Self {
            main_window: Some(main_window),
            event_loop: Some(EventLoop::new()),
        }
    }
    pub fn new_headless(headless: bool, main_window: AmbientWindowBuilder) -> Self {
        Self {
            main_window: Some(main_window),
            event_loop: if headless {
                None
            } else {
                Some(EventLoop::new())
            },
        }
    }

    pub fn simple() -> Self {
        Self::new(AmbientWindowBuilder::new().examples_systems(true))
    }
    pub fn simple_ui() -> Self {
        Self::new(
            AmbientWindowBuilder::new()
                .ui_renderer(true)
                .main_renderer(false)
                .examples_systems(true),
        )
    }
    pub fn simple_dual() -> Self {
        Self::new(
            AmbientWindowBuilder::new()
                .ui_renderer(true)
                .main_renderer(true),
        )
    }

    pub async fn build(
        self,
        init: impl FnOnce(&mut AmbientWindow),
    ) -> anyhow::Result<(App, Option<WindowId>)> {
        #[cfg(feature = "profile")]
        let puffin_server = {
            let puffin_addr = format!(
                "0.0.0.0:{}",
                std::env::var("PUFFIN_PORT")
                    .ok()
                    .and_then(|port| port.parse::<u16>().ok())
                    .unwrap_or(puffin_http::DEFAULT_PORT)
            );
            match puffin_http::Server::new(&puffin_addr) {
                Ok(server) => {
                    tracing::debug!("Puffin server running on {}", puffin_addr);
                    puffin::set_scopes_on(true);
                    Some(server)
                }
                Err(err) => {
                    tracing::error!("Failed to start puffin server: {:?}", err);
                    None
                }
            }
        };

        #[cfg(not(target_os = "unknown"))]
        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);

        let mut app = App {
            event_loop: self.event_loop,
            windows: HashMap::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
        };

        let wind_id = if let Some(window) = self.main_window {
            let mut window = window.build(&mut app).await?;
            init(&mut window);
            let id = window.id();
            app.windows.insert(window.id(), window);
            Some(id)
        } else {
            None
        };
        Ok((app, wind_id))
    }

    /// Runs the app by blocking the main thread
    #[cfg(not(target_os = "unknown"))]
    pub fn block_on(self, init: impl for<'x> AsyncInit<'x>) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let (mut app, window) = self.build(|_| {}).await.unwrap();
            if let Some(window) = window {
                let mut window = app.windows.get_mut(&window).unwrap();
                init.call(&mut window).await
            }

            app.run_blocking();
        });
    }

    /// Finalizes the app and enters the main loop
    pub async fn run(self, init: impl FnOnce(&mut AmbientWindow, RuntimeHandle)) {
        let (app, _) = self
            .build(move |window| {
                let runtime = window.runtime.clone();
                init(window, runtime);
            })
            .await
            .unwrap();
        app.run_blocking()
    }

    #[inline]
    pub async fn run_world(self, init: impl FnOnce(&mut World)) {
        self.run(|app, _| init(&mut app.world)).await
    }
}

pub struct App {
    pub(crate) event_loop: Option<EventLoop<()>>,
    pub(crate) windows: HashMap<WindowId, AmbientWindow>,
    #[cfg(feature = "profile")]
    _puffin: Option<puffin_http::Server>,
}
impl App {
    pub fn is_headless(&self) -> bool {
        self.event_loop.is_none()
    }
    pub fn run_blocking(mut self) {
        if let Some(event_loop) = self.event_loop.take() {
            event_loop.run(move |event, _, control_flow| {
                // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
                // but https://github.com/rust-windowing/winit/issues/1968 restricts us
                if let Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            scale_factor,
                        },
                } = &event
                {
                    *self
                        .windows
                        .get_mut(window_id)
                        .unwrap()
                        .world
                        .resource_mut(window_scale_factor()) = *scale_factor;
                    self.handle_static_event(
                        &Event::WindowEvent {
                            window_id: *window_id,
                            event: WindowEvent::Resized(**new_inner_size),
                        },
                        control_flow,
                    );
                } else if let Some(event) = event.to_static() {
                    self.handle_static_event(&event, control_flow);
                }
            });
        } else {
            // Fake event loop in headless mode
            loop {
                let mut control_flow = ControlFlow::default();
                self.handle_static_event(&Event::MainEventsCleared, &mut control_flow);
                if control_flow == ControlFlow::Exit {
                    return;
                }
            }
        }
    }

    pub fn handle_static_event(
        &mut self,
        event: &Event<'static, ()>,
        control_flow: &mut ControlFlow,
    ) {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                for window in self.windows.values_mut() {
                    window.handle_main_events_cleared(control_flow);
                }
            }
            Event::DeviceEvent { device_id, event } => {
                for window in self.windows.values_mut() {
                    window.handle_device_event(*device_id, event);
                }
            }

            Event::WindowEvent { event, window_id } => {
                self.windows
                    .get_mut(window_id)
                    .unwrap()
                    .handle_event(event, control_flow);
            }
            _ => {}
        }
    }
}

pub trait AsyncInit<'a> {
    type Future: 'a + Future<Output = ()>;
    fn call(self, window: &'a mut AmbientWindow) -> Self::Future;
}

impl<'a, F, Fut> AsyncInit<'a> for F
where
    Fut: 'a + Future<Output = ()>,
    F: FnOnce(&'a mut AmbientWindow) -> Fut,
{
    type Future = Fut;

    fn call(self, window: &'a mut AmbientWindow) -> Self::Future {
        (self)(window)
    }
}
