use crate::{
    visual::{
        aabb_tree::{self, Node},
        graphics::{PushConstants, PUSH_CONSTANT_RANGE},
    },
    Solid, SOLIDS_FRAGMENT, WHOLECANVAS_VERTEX,
};
use std::mem;
use wgpu::*;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const NO_OFFSET: u64 = 0;

const MAX_SOLIDS: u64 = 100;

pub struct SolidsRenderer {
    solids: Vec<Solid>,
    tree: Vec<Node>,

    solids_buffer: Buffer,
    tree_buffer: Buffer,

    pipeline: RenderPipeline,
    bind_group: BindGroup,
}

fn create_buffer<T: bytemuck::Pod>(device: &Device, label: &str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: size * mem::size_of::<T>() as u64,
        usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
        mapped_at_creation: false,
    })
}

impl SolidsRenderer {
    pub fn new(device: &Device) -> Self {
        // Buffers
        let solids_buffer = create_buffer::<Solid>(device, "The solids buffer", MAX_SOLIDS);
        let tree_buffer = create_buffer::<Node>(device, "The AABB tree buffer", 2 * MAX_SOLIDS - 1);

        // Bind group entries for the buffers
        let bind_group_entries = &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &tree_buffer,
                    offset: NO_OFFSET,
                    size: None,
                },
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer {
                    buffer: &solids_buffer,
                    offset: NO_OFFSET,
                    size: None,
                },
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
    pub fn push_to_gpu_buffers(&self, queue: &Queue) {
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
    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, push_constants: PushConstants) {
        pass.set_pipeline(&self.pipeline);
        pass.set_push_constants(
            ShaderStage::FRAGMENT,
            NO_OFFSET as u32,
            bytemuck::cast_slice(&[push_constants]),
        );
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}

fn build_bind_group_and_pipeline(
    device: &Device,
    bind_group_entries: &[BindGroupEntry<'_>],
) -> (BindGroup, RenderPipeline) {
    let bind_group_layout = build_bind_group_layout(device);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("The SolidsRenderer bind group"),
        layout: &bind_group_layout,
        entries: bind_group_entries,
    });

    let pipeline = build_pipeline(device, &bind_group_layout);

    (bind_group, pipeline)
}

fn build_bind_group_layout(device: &Device) -> BindGroupLayout {
    return device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("The solid buffers layout"),
        entries: &[entry(0), entry(1)],
    });

    fn entry(i: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: i,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO Revisit when I understand
            },
            count: None, // Only applicable to sampled textures
        }
    }
}

fn build_pipeline(device: &Device, bind_group_layout: &BindGroupLayout) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("The SolidsRenderer pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[PUSH_CONSTANT_RANGE],
    });

    let vertex_module = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("The wholecanvas vertex shader created in the solids renderer"),
        source: util::make_spirv(WHOLECANVAS_VERTEX),
        flags: ShaderFlags::VALIDATION,
    });
    let fragment_module = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("The solids fragment shader created in the solids renderer"),
        source: util::make_spirv(SOLIDS_FRAGMENT),
        flags: ShaderFlags::VALIDATION,
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("The SolidsRenderer pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &vertex_module,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &fragment_module,
            entry_point: "main",
            targets: &[ColorTargetState {
                format: TEXTURE_FORMAT,
                alpha_blend: BlendState::REPLACE,
                color_blend: BlendState::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
            strip_index_format: None,  // Fine? We do not use index buffers
            front_face: FrontFace::Cw, // Not used since we do not cull
            cull_mode: CullMode::None,
            polygon_mode: PolygonMode::Fill,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            // No multisampling
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}
