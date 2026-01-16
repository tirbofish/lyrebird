use lyrebird_renderer::prelude::*;

pub struct Runtime {

}

impl AppBehaviour for Runtime {
    fn new() -> Self {
        Self {

        }
    }

    fn init(&mut self, ctx: Context) {
        ctx.graphics.window.set_title("lyrebird runtime");
    }

    fn update(&mut self, _ctx: Context, _dt: f64) {
        
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
                multiview_mask: None,
            });
        }

        ctx.graphics.queue.submit(std::iter::once(encoder.finish()));
    }
}