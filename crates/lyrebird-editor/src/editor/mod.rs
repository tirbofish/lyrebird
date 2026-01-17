use lyrebird_renderer::prelude::{winit::keyboard::KeyCode, *};

impl AppBehaviour for crate::LyrebirdEditor {
    fn new() -> Self {
        Self::new().unwrap()
    }

    fn init(&mut self, _ctx: Context) {
        // ctx.graphics.window.set_title("lyrebird editor");
    }
    
    fn update(&mut self, ctx: Context, _dt: f64) {
        if ctx.input.is_key_down(KeyCode::Escape) 
            || ctx.input.gamepads_snapshot().gamepads.iter().find(|(_, state)| state.buttons_down.contains(&gilrs::Button::Start)).is_some()
        {
            
        }
    }
    
    fn render(&mut self, ctx: Context, view: &wgpu::TextureView) {
        let mut encoder = ctx.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        ctx.graphics.queue.submit(std::iter::once(encoder.finish()));
    }
}