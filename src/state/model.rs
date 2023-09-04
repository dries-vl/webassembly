
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // need bytemuck to cast to &[u8] for buffer
pub struct Vertex {                                                  // Pod = plain old data = can convert to u8
    position: [f32; 3],
    tex_coords: [f32; 2]
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // shader skips over this many bytes for next vertex
            step_mode: wgpu::VertexStepMode::Vertex, // per-vertex <=> per-instance data
            attributes: &[ // fields of vertex object
                wgpu::VertexAttribute {
                    offset: 0, // first in memory -> no jump required to get
                    shader_location: 0, // corresponds to @location(0) some_name: vec3<f32>
                    format: wgpu::VertexFormat::Float32x3, // corresponding shader format; FYI: MAX SIZE IS 32x4
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // offset as in jump over the previous elements
                    shader_location: 1, // corresponds to @location(1) some_name: vec3<f32>
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // 0
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // 1
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // 2
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // 3
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // 4
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];


pub struct Instance { // actual rotation and position
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        // Build the model matrix by combining translation and rotation
        let translation_matrix = glam::Mat4::from_translation(self.position);
        let rotation_matrix = glam::Mat4::from_quat(self.rotation);

        let model_matrix = translation_matrix * rotation_matrix;

        // Convert the model matrix to the InstanceRaw representation
        InstanceRaw {
            model: model_matrix.to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw { // converted to shader-usable matrix of position + rotation
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
