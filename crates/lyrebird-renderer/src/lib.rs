use std::{sync::Arc, time::{Duration, Instant}};

use winit::{application::ApplicationHandler, event::{WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::Window};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;

use crate::{input::InputManager, scene::{AppBehaviour, Context}};

mod scene;
mod input;

pub mod prelude {
    pub use super::scene::*;
    pub use super::input::*;

    pub use wgpu;
    pub use winit;
}

/// A version of [State] that can be passed around thread-safe.  
pub struct GraphicsContext {
    pub window: Arc<Window>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub struct State {
    surface: wgpu::Surface<'static>,
    ctx: Arc<GraphicsContext>,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    input_manager: InputManager,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let ctx = Arc::new(GraphicsContext {
            window,
            device: Arc::new(device),
            queue: Arc::new(queue),
        });

        Ok(Self {
            surface,
            ctx,
            config,
            is_surface_configured: false,
            input_manager: InputManager::default(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.ctx.device, &self.config);
            self.is_surface_configured = true;
        }
    }
}

pub struct App<T> {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
    elapsed: Duration,

    instance: T,
}

impl<T> App<T>
where
    T: AppBehaviour,
{
    #[cfg(target_arch = "wasm32")]
    pub fn new(event_loop: &EventLoop<State>) -> Self {
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
            elapsed: Default::default(),
            instance: T::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            state: None,
            elapsed: Default::default(),
            instance: T::new(),
        }
    }
}

impl<T> ApplicationHandler<State> for App<T>
where
    T: AppBehaviour,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            
            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::scene::Context;

            let state = pollster::block_on(State::new(window)).unwrap();
            self.instance.init(Context {
                graphics: state.ctx.clone(),
                input: state.input_manager.clone(),
                event_loop,
            });
            self.state = Some(state);
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            State::new(window)
                                .await
                                .expect("Unable to create canvas!!!")
                        )
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.ctx.window.request_redraw();
            let size = event.ctx.window.inner_size();
            event.resize(size.width, size.height);
        }
        self.instance.init(
            Context {
                graphics: event.ctx.clone(),
                input: event.input_manager.clone(),
                event_loop,
            }
        );
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        if InputManager::is_input_event(&event) {
            state.input_manager.poll(event.clone());
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                self.instance.update(
                    Context {
                        graphics: state.ctx.clone(),
                        input: state.input_manager.clone(),
                        event_loop,
                    }, 
                    self.elapsed.as_secs_f64()
                );

                let mut render = || -> Result<(), wgpu::SurfaceError> {
                    state.ctx.window.request_redraw();

                    if !state.is_surface_configured {
                        return Ok(());
                    }
                    
                    let output = state.surface.get_current_texture()?;
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    self.instance.render(
                        Context {
                            graphics: state.ctx.clone(),
                            input: state.input_manager.clone(),
                            event_loop,
                        }, 
                        &view
                    );

                    output.present();

                    Ok(())
                };

                match render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.ctx.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
                self.elapsed = now.elapsed();
            }
            _ => {}
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.instance.exiting(event_loop);
        log::info!("Exiting");
    }
}

pub fn run<T>() -> anyhow::Result<()> 
where T: AppBehaviour
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;

        let app = App::<T>::new(&event_loop);
        event_loop.spawn_app(app);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::<T>::new();
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}