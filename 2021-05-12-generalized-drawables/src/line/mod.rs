use crate::layer::{Drawable, Layer};
use wgpu::util::DeviceExt;
use wgpu::{
    BlendComponent, BlendState, Buffer, Device, RenderPass, RenderPipeline, SwapChainDescriptor,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Line {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
    pub width: f32,
}

pub struct LinesLayer {
    data: Vec<Line>,
}

impl LinesLayer {
    pub fn new(data: Vec<Line>) -> Self {
        LinesLayer { data }
    }
}

pub struct LinesLayerDrawable {
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
    num_lines: u32,
}

impl Drawable for LinesLayerDrawable {
    fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.num_lines);
    }
}

impl Layer for LinesLayer {
    fn init_drawable(&self, device: &Device, sc_desc: &SwapChainDescriptor) -> Box<dyn Drawable> {
        let instance_buffer_desc = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Line>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&self.data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[instance_buffer_desc],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    write_mask: wgpu::ColorWrite::ALL,
                    blend: Some(BlendState {
                        color: BlendComponent::OVER,
                        alpha: BlendComponent::REPLACE,
                    }),
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                //cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Box::new(LinesLayerDrawable {
            render_pipeline,
            instance_buffer,
            num_lines: self.data.len() as u32,
        })
    }
}
