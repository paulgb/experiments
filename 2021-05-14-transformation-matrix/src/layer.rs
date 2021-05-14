use wgpu::{Device, RenderPass, SwapChainDescriptor, BindGroupLayout, BindGroup};

pub trait Layer {
    fn init_drawable(&self, device: &Device, sc_desc: &SwapChainDescriptor, transform_layout: &BindGroupLayout) -> Box<dyn Drawable>;
}

pub trait Drawable {
    fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, bind_group: &'a BindGroup);
}
