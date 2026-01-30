//! GPU setup and rendering

use super::particle::Particle;
use crate::types::{Color, ConfettiOptions, Origin};
use std::sync::mpsc::Receiver;
use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub enum Command {
    Fireworks,
    Celebration,
}

pub fn run_event_loop(rx: Receiver<Command>) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    if let Some(monitor) = window.current_monitor() {
        let size = monitor.size();
        let pos = monitor.position();
        window.set_outer_position(winit::dpi::PhysicalPosition::new(
            pos.x + (size.width as i32 - 800) / 2,
            pos.y + (size.height as i32 - 600) / 2,
        ));
    }

    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .unwrap();

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Particle Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let format = surface.get_capabilities(&adapter).formats[0];

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut particles: Vec<Particle> = Vec::new();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            if let Ok(cmd) = rx.try_recv() {
                spawn(cmd, &mut particles);
                window.request_redraw();
            }

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => elwt.exit(),
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    if particles.is_empty() {
                        return;
                    }
                    render(&surface, &device, &queue, &pipeline, &mut particles);
                }
                _ => {}
            }
        })
        .unwrap();
}

fn spawn(cmd: Command, particles: &mut Vec<Particle>) {
    particles.clear();
    let opts = match cmd {
        Command::Fireworks => ConfettiOptions {
            particle_count: 100,
            spread: 360.0,
            start_velocity: 30.0,
            gravity: 0.5,
            origin: Origin { x: 0.5, y: 0.5 },
            ..Default::default()
        },
        Command::Celebration => ConfettiOptions {
            particle_count: 50,
            angle: 60.0,
            spread: 55.0,
            origin: Origin { x: 0.0, y: 0.6 },
            ..Default::default()
        },
    };

    for i in 0..opts.particle_count {
        let color = opts.colors[rand::random::<usize>() % opts.colors.len()];
        let mut p = Particle::new(&opts, 400.0, 300.0, color);

        if matches!(cmd, Command::Celebration) {
            if i % 2 == 0 {
                p.x = 0.0;
                p.vx = p.vx.abs();
            } else {
                p.x = 800.0;
                p.vx = -p.vx.abs();
            }
        }
        particles.push(p);
    }
}

fn render(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    particles: &mut Vec<Particle>,
) {
    particles.retain_mut(|p| p.update(0.5, 0.0));
    if particles.is_empty() {
        return;
    }

    let output = surface.get_current_texture().unwrap();
    let view = output.texture.create_view(&Default::default());

    let data: Vec<[f32; 6]> = particles
        .iter()
        .map(|p| [p.x, p.y, p.color[0], p.color[1], p.color[2], p.color[3]])
        .collect();

    let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let verts: &[f32] = &[
        -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
    ];
    let vert_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(verts),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_vertex_buffer(0, vert_buf.slice(..));
        pass.set_vertex_buffer(1, instance_buf.slice(..));
        pass.draw(0..6, 0..particles.len() as u32);
    }
    queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
