use core::panic;
use std::{borrow::Cow, convert::TryInto};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BackendBit, BindGroupDescriptor, BindGroupEntry, BufferAddress, BufferDescriptor, BufferUsage,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Features, Limits, RequestAdapterOptions, ShaderFlags, ShaderModuleDescriptor, ShaderSource,
};

async fn run() {
    let input: Vec<f32> = vec![0.0];

    let instance = wgpu::Instance::new(BackendBit::PRIMARY);
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                label: None,
                limits: Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let flags = wgpu::ShaderFlags::VALIDATION | ShaderFlags::EXPERIMENTAL_TRANSLATION;

    let c = include_bytes!("out.spv");
    let b: &[u32; 142] = unsafe { &core::mem::transmute(*c) };

    let cs_module = device.create_shader_module(&ShaderModuleDescriptor {
        flags,
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        //source: ShaderSource::SpirV(Cow::Borrowed(b)),
    });

    let slice_size = input.len() * std::mem::size_of::<f32>();
    let size = slice_size as BufferAddress;

    let staging_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        size,
        usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
    });

    let storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
        contents: bytemuck::cast_slice(&input),
        label: None,
        usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    });

    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        entry_point: "main",
        module: &cs_module,
        layout: None,
        label: None,
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

    {
        let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute");
        cpass.dispatch(1, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);

    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    device.poll(wgpu::Maintain::Wait);

    let result = if let Ok(()) = buffer_future.await {
        let data = buffer_slice.get_mapped_range();
        let result: Vec<f32> = data
            .chunks_exact(4)
            .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
            .collect();

        drop(data);
        staging_buffer.unmap();
        result
    } else {
        panic!("failed to run compute");
    };

    println!("Result: {:?}", &result);
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
