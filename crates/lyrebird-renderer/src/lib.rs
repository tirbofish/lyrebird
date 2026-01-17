use std::sync::Arc;

use slint::{ComponentHandle, wgpu_27::{WGPUConfiguration, WGPUSettings}};
use wgpu::{Extent3d, Instance, TextureDescriptor};

use crate::{input::InputManager, scene::{AppBehaviour, Context}};

mod scene;
mod input;

pub mod prelude {
    pub use super::scene::*;
    pub use super::input::*;

    pub use wgpu;
    pub use winit;
    #[cfg(not(target_arch = "wasm32"))]
    pub use gilrs;
}

/// A version of [State] that can be passed around thread-safe.  
pub struct GraphicsContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub struct State {
    #[allow(dead_code)]
    instance: Instance,
    ctx: Arc<GraphicsContext>,
    input_manager: InputManager,
}

impl State {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
}

pub fn run<S>() -> anyhow::Result<()> 
where 
    S: ComponentHandle + AppBehaviour + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    slint::BackendSelector::new()
        .require_wgpu_27(WGPUConfiguration::Automatic(WGPUSettings::default()))
        .select()
        .expect("Unable to create Slint backend with WGPU based renderer");

    let slint_app = S::new();

    let mut last_frame = std::time::Instant::now();
    let mut offscreen_texture: Option<wgpu::Texture> = None;
    let mut old_size = slint_app.window().size();
    let mut renderer = None;
    let mut app = slint_app.clone_strong();
    slint_app.window().set_rendering_notifier(move |state, api| {
        match state {
            slint::RenderingState::RenderingSetup => {
                if let slint::GraphicsAPI::WGPU27 { instance, device, queue, .. } = api {
                    let ctx = GraphicsContext {
                        device:  Arc::new(device.clone()),
                        queue: Arc::new(queue.clone()),
                    };

                    let state = State {
                        instance: instance.clone(),
                        ctx: Arc::new(ctx),
                        input_manager: InputManager::default(),
                    };

                    renderer = Some(state);
                }
            },
            slint::RenderingState::BeforeRendering => {
                if let Some(state) = &renderer {
                    // use i_slint_backend_winit::WinitWindowAccessor;

                    let now = std::time::Instant::now();
                    let dt = now.duration_since(last_frame).as_secs_f64();
                    last_frame = now;

                    // if InputManager::is_input_event(&event) {
                    //     state.input_manager.poll(event.clone());
                    // }

                    state.input_manager.update_gamepads();

                    app.update(
                        Context { 
                            graphics: state.ctx.clone(), 
                            input: state.input_manager.clone(),
                        },
                        dt
                    );

                    let size = app.window().size();
                    let width = size.width;
                    let height = size.height;

                    if offscreen_texture.is_none() || old_size != size {
                        old_size = size;
                        offscreen_texture = Some(state.ctx.device.create_texture(&TextureDescriptor {
                            label: Some("viewport texture"),
                            size: Extent3d {
                                width: width.max(1),
                                height: height.max(1),
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: State::FORMAT,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        }));
                    }
                    let texture = offscreen_texture.as_ref().unwrap();
                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

                    app.render(
                        Context {
                            graphics: state.ctx.clone(),
                            input: state.input_manager.clone(),
                        },
                        &view
                    );

                    // app.set_texture(slint::Image::try_from(texture.clone()).unwrap());

                    app.window().request_redraw();
                }

                app.window().request_redraw();
            }
            slint::RenderingState::AfterRendering => {},
            slint::RenderingState::RenderingTeardown => {
                if let Some(state) = &renderer {
                    app.exiting(Context {
                        graphics: state.ctx.clone(),
                        input: state.input_manager.clone(),
                    });
                    log::info!("Exiting app");
                }
                drop(renderer.take());
            },
            _ => todo!(),
        }
    }).unwrap();

    Ok(slint_app.run()?)
}