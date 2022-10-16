use crate::{
    visual::{
        aabb_tree::{self, Node},
        graphics::{PushConstants, PUSH_CONSTANT_RANGE},
    },
    Solid, SOLIDS_FRAGMENT, WHOLECANVAS_VERTEX,
};
use std::mem;

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const NO_OFFSET: u64 = 0;

const MAX_SOLIDS: u64 = 100;

pub struct SolidsRenderer {
    solids: Vec<Solid>,
    tree: Vec<Node>,

    solids_buffer: wgpu::Buffer,
    tree_buffer: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

fn create_buffer<T: bytemuck::Pod>(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size * mem::size_of::<T>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl SolidsRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        // Buffers
        let solids_buffer = create_buffer::<Solid>(device, "The solids buffer", MAX_SOLIDS);
        let tree_buffer = create_buffer::<Node>(device, "The AABB tree buffer", 2 * MAX_SOLIDS - 1);

        // Bind group entries for the buffers
        let bind_group_entries = &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &tree_buffer,
                    offset: NO_OFFSET,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &solids_buffer,
                    offset: NO_OFFSET,
                    size: None,
                }),
            },
        ];

        let (bind_group, pipeline) = build_bind_group_and_pipeline(device, bind_group_entries);

        Self {
            solids: Vec::new(),
            tree: Vec::new(),

            solids_buffer,
            tree_buffer,

            pipeline,
            bind_group,
        }
    }
    pub fn update(&mut self, solids: Vec<Solid>) {
        for solid in &solids {
            solid.assert_valid();
        }

        let tree = aabb_tree::build_tree(&solids);

        assert_ne!(tree.len(), 0);
        assert_eq!(tree.len(), 2 * solids.len() - 1);

        self.solids = solids;
        self.tree = tree;
    }
    pub fn push_to_gpu_buffers(&self, queue: &wgpu::Queue) {
        assert_ne!(self.tree.len(), 0);
        assert_eq!(self.tree.len(), 2 * self.solids.len() - 1);

        queue.write_buffer(
            &self.solids_buffer,
            NO_OFFSET,
            bytemuck::cast_slice(&self.solids),
        );
        queue.write_buffer(
            &self.tree_buffer,
            NO_OFFSET,
            bytemuck::cast_slice(&self.tree),
        );
    }
    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, push_constants: PushConstants) {
        pass.set_pipeline(&self.pipeline);
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            NO_OFFSET as u32,
            bytemuck::cast_slice(&[push_constants]),
        );
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}

fn build_bind_group_and_pipeline(
    device: &wgpu::Device,
    bind_group_entries: &[wgpu::BindGroupEntry<'_>],
) -> (wgpu::BindGroup, wgpu::RenderPipeline) {
    let bind_group_layout = build_bind_group_layout(device);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("The SolidsRenderer bind group"),
        layout: &bind_group_layout,
        entries: bind_group_entries,
    });

    let pipeline = build_pipeline(device, &bind_group_layout);

    (bind_group, pipeline)
}

fn build_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    return device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("The solid buffers layout"),
        entries: &[entry(0), entry(1)],
    });

    fn entry(i: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO Revisit when I understand
            },
            count: None, // Only applicable to sampled textures
        }
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("The SolidsRenderer pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[PUSH_CONSTANT_RANGE],
    });

    let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("The wholecanvas vertex shader created in the solids renderer"),
        source: wgpu::util::make_spirv(WHOLECANVAS_VERTEX),
    });
    let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("The solids fragment shader created in the solids renderer"),
        source: wgpu::util::make_spirv(SOLIDS_FRAGMENT),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("The SolidsRenderer pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_module,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_module,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None, // Fine? We do not use index buffers
            front_face: wgpu::FrontFace::Cw, // Not used since we do not cull
            unclipped_depth: false,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            // No multisampling
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
