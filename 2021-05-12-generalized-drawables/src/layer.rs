use wgpu::{Device, RenderPass, SwapChainDescriptor};

pub trait Layer {
    type D: Drawable;

    fn init_drawable(&self, device: &Device, sc_desc: &SwapChainDescriptor) -> Self::D;
}

pub trait Drawable {
    fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>);
}
