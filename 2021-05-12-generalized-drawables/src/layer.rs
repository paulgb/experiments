use wgpu::{Device, RenderPass, SwapChainDescriptor};

pub trait Layer {
    fn init_drawable(&self, device: &Device, sc_desc: &SwapChainDescriptor) -> Box<dyn Drawable>;
}

pub trait Drawable {
    fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>);
}
