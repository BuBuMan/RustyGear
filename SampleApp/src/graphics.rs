use wgpu::util::DeviceExt;

#[path = "texture.rs"]
pub mod texture;

use texture::Texture;

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

pub struct Graphics {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: (u32, u32),
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer, 
    pub num_indices: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl Graphics {
    pub async fn new(window: &sdl2::video::Window) -> Self {
        let size = window.size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // Surface is used to create the swap chain and adapter
        let surface = unsafe { instance.create_surface(window) };

        // Adapter is used to create the device and queue
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
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
        let diffuse_texture = Texture::load_texture("src\\resources\\textures\\spaceship.png", &device, &queue).unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        });

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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

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
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.0;
        self.swap_chain_descriptor.height = new_size.1;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}