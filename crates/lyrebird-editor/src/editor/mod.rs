use lyrebird_renderer::prelude::*;

pub struct Editor {

}

impl AppBehaviour for Editor {
    fn new() -> Self {
        Self {

        }
    }

    fn init(&mut self, ctx: std::sync::Arc<lyrebird_renderer::GraphicsContext>) {
        ctx.window.set_title("lyrebird editor");
    }

    fn update(&mut self, _ctx: std::sync::Arc<lyrebird_renderer::GraphicsContext>, _dt: f64) {
    }

    fn render(&mut self, _ctx: std::sync::Arc<lyrebird_renderer::GraphicsContext>, _view: &wgpu::TextureView) {
    }
}