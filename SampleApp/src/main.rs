#![allow(non_snake_case)]

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use std::collections::HashSet;
use futures::executor::block_on;
use wgpu::util::DeviceExt;

mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn Desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

// A--B
// |  |
// C--D
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] },  // A
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] },   // B
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // C
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] },  // D
];

// Must be 4 bytes aligned
const INDICES: &[u16] = &[
    2, 1, 0,
    1, 2, 3,
];

struct Graphics {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: (u32, u32),
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer, 
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
}

impl Graphics {
    async fn new(window: &sdl2::video::Window) -> Self {
        let size = window.size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // Surface is used to create the swap chain and adapter
        let surface = unsafe { instance.create_surface(window) };

        // Adapter is used to create the device and queue
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                // Specify any extra gpu feature. You can get a list of features supported by your device using adapter.features(), or device.features().
                // https://docs.rs/wgpu/0.7.0/wgpu/struct.Features.html
                features: wgpu::Features::empty(),

                // The limits field describes the limit of certain types of resource we can create.
                // https://docs.rs/wgpu/0.7.0/wgpu/struct.Limits.html
                limits: wgpu::Limits::default(),


                label: None,
            },
            None,
        ).await.unwrap();
        
        // Define and creating the swap_chain.
        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            // The usage field describes how the swap_chain's underlying textures will be used. 
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            // Defines how the swap_chains textures will be stored on the gpu
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.0,
            height: size.1,
            // The present_mode uses the wgpu::PresentMode enum which determines how to sync the swap chain with the display. 
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        // Create texture
        let diffuse_texture = texture::Texture::load_texture("src\\resources\\textures\\spaceship.png", &device, &queue).unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        // Load shders
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("resources\\shaders\\shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::Desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swap_chain_descriptor.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // To select alpha
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size,
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
        }
    }

    // impl State
    fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.0;
        self.swap_chain_descriptor.height = new_size.1;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    fn render(&mut self, game_state: &GameState) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(game_state.clear_color),
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        // Finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

struct Input {
    current_pressed_keys: HashSet<sdl2::keyboard::Scancode>,
    previous_pressed_keys: HashSet<sdl2::keyboard::Scancode>,
}

impl Input {
    pub fn new(eventPump: &sdl2::EventPump) -> Self {
        Self {
            current_pressed_keys: eventPump.keyboard_state().pressed_scancodes().collect(),
            previous_pressed_keys: eventPump.keyboard_state().pressed_scancodes().collect()
        }
    }

    pub fn update(&mut self, newKeyboardState: &sdl2::keyboard::KeyboardState) {
        std::mem::swap(&mut self.current_pressed_keys, &mut self.previous_pressed_keys);
        self.current_pressed_keys = newKeyboardState.pressed_scancodes().collect();
    }

    pub fn is_key_pressed(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.current_pressed_keys.contains(&key)
    }

    pub fn is_key_down(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.current_pressed_keys.contains(&key) && !self.previous_pressed_keys.contains(&key)
    }

    pub fn is_key_up(&self, key: sdl2::keyboard::Scancode) -> bool {
        !self.current_pressed_keys.contains(&key) && self.previous_pressed_keys.contains(&key)
    }
}

struct AppState {
    input: Input,
    graphics: Graphics,
    start_of_frame: Instant,
    time_elapsed: f64,
    delta_time: f64,
    target_fps: u16,
    exit_app: bool
}

struct GameState {
    clear_color: wgpu::Color
}

impl AppState {
    pub fn new(input: Input, graphics: Graphics, target_fps: Option<u16>) -> AppState {
        let fps = target_fps.unwrap_or(60);
        AppState {
            input: input,
            graphics: graphics,
            start_of_frame: Instant::now(),
            time_elapsed: 0.0,
            target_fps: fps,
            delta_time: 1.0/(fps as f64),
            exit_app: false
        }
    }

    pub fn TargetRefreshRate(&self) -> f64 {
        1.0/(self.target_fps as f64)
    }
}

fn update_game(appState: &AppState, game_state: &mut GameState) {
    if appState.input.is_key_down(sdl2::keyboard::Scancode::R) {
        game_state.clear_color.r = 1.0;
        game_state.clear_color.g = 0.0;
        game_state.clear_color.b = 0.0;
    }

    if appState.input.is_key_down(sdl2::keyboard::Scancode::G) {
        game_state.clear_color.r = 0.0;
        game_state.clear_color.g = 1.0;
        game_state.clear_color.b = 0.0;
    }

    if appState.input.is_key_down(sdl2::keyboard::Scancode::B) {
        game_state.clear_color.r = 0.0;
        game_state.clear_color.g = 0.0;
        game_state.clear_color.b = 1.0;
    }
}

fn enter_frame(event_pump: &mut sdl2::EventPump, app_state: &mut AppState) {
    app_state.start_of_frame = Instant::now();
    app_state.input.update(&event_pump.keyboard_state());

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>  {
                app_state.exit_app = true;
            },
            Event::Window { win_event : sdl2::event::WindowEvent::Resized(width, height), .. }=> {
                app_state.graphics.resize((width as u32, height as u32));
            },
            _ => {}
        }
    }
}

fn exit_frame(app_state: &mut AppState) {
    'lockFPS: loop {
        if app_state.start_of_frame.elapsed().as_secs_f64() >= app_state.TargetRefreshRate() {
            break 'lockFPS;
        }
    }

    app_state.delta_time = app_state.start_of_frame.elapsed().as_secs_f64();
    app_state.time_elapsed += app_state.delta_time;

    // println!("Application has been running for: {:?} seconds", appState.timeElapsed);
    // println!("Application FPS: {:?} ", 1.0/appState.deltaTime);
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Sample", 800, 800)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let graphics = block_on(Graphics::new(&window));
    let mut app_state = AppState::new(Input::new(&event_pump), graphics, None);
    let mut game_state = GameState {
        clear_color: wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0
        }
    };

    'game_loop: loop {
        enter_frame(&mut event_pump, &mut app_state);

        if app_state.exit_app {
            break 'game_loop;
        }

        update_game(&app_state, &mut game_state);

        match app_state.graphics.render(&game_state) {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => app_state.graphics.resize(app_state.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => app_state.exit_app = true,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        exit_frame(&mut app_state);
    }
}
