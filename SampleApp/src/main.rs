#![allow(non_snake_case)]

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use std::collections::HashSet;
use futures::executor::block_on;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
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
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
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
    num_vertices: u32,
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

        // Load shders
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        let num_vertices = VERTICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size,
            pipeline,
            vertex_buffer,
            num_vertices
        }
    }

    // impl State
    fn Resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.0;
        self.swap_chain_descriptor.height = new_size.1;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    fn Render(&mut self, gameState: &GameState) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut renderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(gameState.clearColor),
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        });

        renderPass.set_pipeline(&self.pipeline);
        renderPass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderPass.draw(0..self.num_vertices, 0..1);

        drop(renderPass);

        // Finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

struct Input {
    currentPressedKeys: HashSet<sdl2::keyboard::Scancode>,
    previousPressedKeys: HashSet<sdl2::keyboard::Scancode>,
}

impl Input {
    pub fn new(eventPump: &sdl2::EventPump) -> Self {
        Self {
            currentPressedKeys: eventPump.keyboard_state().pressed_scancodes().collect(),
            previousPressedKeys: eventPump.keyboard_state().pressed_scancodes().collect()
        }
    }

    pub fn Update(&mut self, newKeyboardState: &sdl2::keyboard::KeyboardState) {
        std::mem::swap(&mut self.currentPressedKeys, &mut self.previousPressedKeys);
        self.currentPressedKeys = newKeyboardState.pressed_scancodes().collect();
    }

    pub fn IsKeyPressed(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.currentPressedKeys.contains(&key)
    }

    pub fn IsKeyDown(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.currentPressedKeys.contains(&key) && !self.previousPressedKeys.contains(&key)
    }

    pub fn IsKeyUp(&self, key: sdl2::keyboard::Scancode) -> bool {
        !self.currentPressedKeys.contains(&key) && self.previousPressedKeys.contains(&key)
    }
}

struct AppState {
    input: Input,
    graphics: Graphics,
    startOfFrame: Instant,
    timeElapsed: f64,
    deltaTime: f64,
    targetFPS: u16,
    exitApp: bool
}

struct GameState {
    clearColor: wgpu::Color
}

impl AppState {
    pub fn new(input: Input, graphics: Graphics, targerFPS: Option<u16>) -> AppState {
        let fps = targerFPS.unwrap_or(60);
        AppState {
            input: input,
            graphics: graphics,
            startOfFrame: Instant::now(),
            timeElapsed: 0.0,
            targetFPS: fps,
            deltaTime: 1.0/(fps as f64),
            exitApp: false
        }
    }

    pub fn TargetRefreshRate(&self) -> f64 {
        1.0/(self.targetFPS as f64)
    }
}

fn UpdateGame(appState: &AppState, gameState: &mut GameState) {
    if appState.input.IsKeyDown(sdl2::keyboard::Scancode::R) {
        gameState.clearColor.r = 1.0;
        gameState.clearColor.g = 0.0;
        gameState.clearColor.b = 0.0;
    }

    if appState.input.IsKeyDown(sdl2::keyboard::Scancode::G) {
        gameState.clearColor.r = 0.0;
        gameState.clearColor.g = 1.0;
        gameState.clearColor.b = 0.0;
    }

    if appState.input.IsKeyDown(sdl2::keyboard::Scancode::B) {
        gameState.clearColor.r = 0.0;
        gameState.clearColor.g = 0.0;
        gameState.clearColor.b = 1.0;
    }
}

fn EnterFrame(eventPump: &mut sdl2::EventPump, appState: &mut AppState) {
    appState.startOfFrame = Instant::now();
    appState.input.Update(&eventPump.keyboard_state());

    for event in eventPump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>  {
                appState.exitApp = true;
            },
            Event::Window { win_event : sdl2::event::WindowEvent::Resized(width, height), .. }=> {
                appState.graphics.Resize((width as u32, height as u32));
            },
            _ => {}
        }
    }
}

fn ExitFrame(appState: &mut AppState) {
    'lockFPS: loop {
        if appState.startOfFrame.elapsed().as_secs_f64() >= appState.TargetRefreshRate() {
            break 'lockFPS;
        }
    }

    appState.deltaTime = appState.startOfFrame.elapsed().as_secs_f64();
    appState.timeElapsed += appState.deltaTime;

    // println!("Application has been running for: {:?} seconds", appState.timeElapsed);
    // println!("Application FPS: {:?} ", 1.0/appState.deltaTime);
}

fn main() {
    let sdlContext = sdl2::init().unwrap();
    let videoSubsystem = sdlContext.video().unwrap();
    let window = videoSubsystem
        .window("Sample", 800, 800)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    let mut eventPump = sdlContext.event_pump().unwrap();

    let graphics = block_on(Graphics::new(&window));
    let mut appState = AppState::new(Input::new(&eventPump), graphics, None);
    let mut gameState = GameState {
        clearColor: wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0
        }
    };

    'gameloop: loop {
        EnterFrame(&mut eventPump, &mut appState);

        if appState.exitApp {
            break 'gameloop;
        }

        UpdateGame(&appState, &mut gameState);

        match appState.graphics.Render(&gameState) {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => appState.graphics.Resize(appState.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => appState.exitApp = true,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        ExitFrame(&mut appState);
    }
}
