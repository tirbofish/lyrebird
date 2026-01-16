use std::sync::Arc;


/// Defines the behaviour of an app. 
pub trait AppBehaviour {
    fn new() -> Self;
    fn init(&mut self, ctx: Arc<crate::GraphicsContext>);
    fn update(&mut self, ctx: Arc<crate::GraphicsContext>, dt: f64);
    fn render(&mut self, ctx: Arc<crate::GraphicsContext>, view: &wgpu::TextureView);
}