use std::collections::HashMap;
use std::fs;
use wgpu::util::DeviceExt;
use crate::texture::Texture;

pub struct Graphics {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: (u32, u32),
    pub models: HashMap<String, Mesh>,
    pub textures: HashMap<String, wgpu::BindGroup>,
    pub pipelines: HashMap<String, wgpu::RenderPipeline>,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelProperties {
    pub model_matrix: [[f32; 4]; 4],
}

fn create_quad() -> Mesh {
    let mut vertices = Vec::new();

    let vertexA = Vertex {
        position: [-0.5, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    };

    let vertexB = Vertex {
        position: [0.5, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
    };

    let vertexC = Vertex {
        position: [-0.5, -0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    };

    let vertexD = Vertex {
        position: [0.5, -0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    };

    vertices.push(vertexA);
    vertices.push(vertexB);
    vertices.push(vertexC);
    vertices.push(vertexD);

    let indices = vec!(2, 1, 0, 1, 2, 3);

    Mesh {
        vertices,
        indices,
        vertex_buffer: None,
        index_buffer: None,
    }
}

impl Mesh {
    fn upload_to_gpu(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));

        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsage::INDEX,
        }));
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],   
}

impl Uniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, matrix4: cgmath::Matrix4<f32>) {
        self.view_proj = matrix4.into();
    }
}

pub fn upload_texture_to_gpu(texture_name: &str, device: &wgpu::Device, queue: &wgpu::Queue, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let texture = Texture::load_texture(texture_name, &device, &queue).unwrap();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            }
        ],
        label: Some(texture_name),
    })
}

pub fn load_shader(shader_name: &str) -> Vec<u8> {
    let mut shader_dir = std::env::current_dir().unwrap();
    shader_dir.push("src\\resources\\shaders");
    shader_dir.push(shader_name);

    match fs::read(&shader_dir) {
        Ok(v) => v,
        Err(error) => panic!("Failed to read the file: {:?}. Error: {}", shader_dir.as_path(), error)
    }
}

pub fn new_pipeline(device: &wgpu::Device, texture_format: wgpu::TextureFormat, vert_shader_name: &str, frag_shader_name: &str, texture_bind_group_layout: &wgpu::BindGroupLayout, uniform_bind_group_layout: &wgpu::BindGroupLayout, topology: wgpu::PrimitiveTopology, polygon_mode: wgpu::PolygonMode) -> wgpu::RenderPipeline {
    let vert_shader_contents = load_shader(vert_shader_name);
    let frag_shader_contents = load_shader(frag_shader_name);
    
    let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some(vert_shader_name),
        flags: wgpu::ShaderFlags::all(),
        source: wgpu::util::make_spirv(&vert_shader_contents),
    });

    let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some(frag_shader_name),
        flags: wgpu::ShaderFlags::all(),
        source: wgpu::util::make_spirv(&frag_shader_contents),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniform_bind_group_layout,
            ],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu_types::ShaderStage::VERTEX,
            range: 0..128,
        }],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: "main",
            buffers: &[Vertex::Desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // To select alpha
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: polygon_mode,
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

    pipeline
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
                features: wgpu::Features::PUSH_CONSTANTS,

                // The limits field describes the limit of certain types of resource we can create.
                // https://docs.rs/wgpu/0.7.0/wgpu/struct.Limits.html
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..wgpu::Limits::default()
                },

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

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let mut pipelines : HashMap::<String, wgpu::RenderPipeline> = HashMap::new();
        let pipeline = new_pipeline(&device, swap_chain_descriptor.format, "sprite.vert.spv", "sprite.frag.spv", &texture_layout, &uniform_bind_group_layout, wgpu::PrimitiveTopology::TriangleList, wgpu::PolygonMode::Fill);
        pipelines.insert("sprite".to_owned(), pipeline);

        let mut models : HashMap::<String, Mesh> = HashMap::new();
        let mut triangle_mesh = create_quad();
        triangle_mesh.upload_to_gpu(&device);
        models.insert("quad".to_owned(), triangle_mesh);

        let mut textures : HashMap<String, wgpu::BindGroup> = HashMap::new();
        textures.insert("spaceship.png".to_owned(), upload_texture_to_gpu("spaceship.png", &device, &queue, &texture_layout));

        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size,
            models,
            textures,
            pipelines,
            texture_layout,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.0;
        self.swap_chain_descriptor.height = new_size.1;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}