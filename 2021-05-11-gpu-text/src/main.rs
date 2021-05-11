use std::error::Error;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use futures::task::SpawnExt;


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let surface = unsafe { instance.create_surface(&window) };

    // Initialize GPU
    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface)
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Request device")
    });

    let mut staging_belt = wgpu::util::StagingBelt::new(1024);
    let mut local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox
        }
    );

    let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "Inconsolata-Regular.ttf"
    ))?;

    let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
       match event {
           // Close.
           Event::WindowEvent {
               event: WindowEvent::CloseRequested,
               ..
           } => *control_flow = ControlFlow::Exit,

           // Resize.
           Event::WindowEvent {
               event: WindowEvent::Resized(new_size),
               ..
           } => {
               size = new_size;
               // TODO: this is repeated
               swap_chain = device.create_swap_chain(
                   &surface,
                   &wgpu::SwapChainDescriptor {
                       usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                       format: render_format,
                       width: size.width,
                       height: size.height,
                       present_mode: wgpu::PresentMode::Mailbox,
                   }
               )
           }

           // Redraw.
           Event::RedrawRequested {..} => {
               // Get a command encoder for the current frame
               let mut encoder = device.create_command_encoder(
                   &wgpu::CommandEncoderDescriptor {
                       label: Some("Redraw")
                   }
               );

               // Get the next frame
               let frame = swap_chain
                   .get_current_frame()
                   .expect("Get next frame")
                   .output;

               // Clear frame
               {
                   let _ = encoder.begin_render_pass(
                       &wgpu::RenderPassDescriptor {
                           label: Some("Render pass"),
                           color_attachments: &[
                               wgpu::RenderPassColorAttachmentDescriptor {
                                   attachment: &frame.view,
                                   resolve_target: None,
                                   ops: wgpu::Operations {
                                       load: wgpu::LoadOp::Clear(
                                           wgpu::Color {
                                               r: 0.4,
                                               g: 0.4,
                                               b: 0.4,
                                               a: 1.0
                                           }
                                       ),
                                       store: true
                                   }
                               }
                           ],
                           depth_stencil_attachment: None
                       }
                   );
               }

               glyph_brush.queue(Section {
                   screen_position: (30., 30.),
                   bounds: (size.width as f32, size.height as f32),
                   text: vec![
                       Text::new("Hello, world!")
                           .with_color([0.0, 0.0, 0.0, 1.0])
                           .with_scale(40.0),
                   ],
                   ..Section::default()
               });

               glyph_brush.draw_queued(
                   &device, &mut staging_belt, &mut encoder,
                   &frame.view, size.width, size.height
               ).expect("Draw queued");

               staging_belt.finish();
               queue.submit(Some(encoder.finish()));

               local_spawner
                   .spawn(staging_belt.recall())
                   .expect("Recall staging belt");

               local_pool.run_until_stalled();
           }


           _ => *control_flow = ControlFlow::Wait
       }
    });

}
