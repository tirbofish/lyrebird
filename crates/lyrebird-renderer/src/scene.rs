use std::sync::Arc;

use crate::input::InputManager;

pub struct Context {
    pub graphics: Arc<crate::GraphicsContext>,
    pub input: InputManager,
}

/// Defines the behaviour of an app. 
pub trait AppBehaviour {
    fn new() -> Self;
    fn init(&mut self, ctx: Context);
    fn update(&mut self, ctx: Context, dt: f64);
    fn render(&mut self, ctx: Context, view: &wgpu::TextureView);

    fn exiting(&mut self, _ctx: Context) {}
}