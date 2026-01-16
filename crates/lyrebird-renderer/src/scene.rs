use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;

use crate::input::InputManager;

pub struct Context<'a> {
    pub graphics: Arc<crate::GraphicsContext>,
    pub input: InputManager,
    pub event_loop: &'a ActiveEventLoop,
}

/// Defines the behaviour of an app. 
pub trait AppBehaviour {
    fn new() -> Self;
    fn init(&mut self, ctx: Context);
    fn update(&mut self, ctx: Context, dt: f64);
    fn render(&mut self, ctx: Context, view: &wgpu::TextureView);

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}