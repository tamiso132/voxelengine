use std::mem;

use ash::vk;
use glm::{Vec2, Vec3};

pub trait Vertex {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription>;
    fn get_vertex_binding_desc() -> Vec<vk::VertexInputBindingDescription>
    where
        Self: Sized,
    {
        [vk::VertexInputBindingDescription::default().binding(0).stride(mem::size_of::<Self>() as u32).input_rate(vk::VertexInputRate::VERTEX)].to_vec()
    }
}
#[repr(u32)]
pub enum Face {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back,
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct ChunkMesh {
    pub pos: glm::Vec3,
    norm: glm::Vec3,
    uv: glm::Vec2, // will repeat
    texture_index: u32,
}

impl Vertex for ChunkMesh {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        todo!()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, align(16))]
pub struct EmptyVertex;

impl Vertex for EmptyVertex {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![]
    }

    fn get_vertex_binding_desc() -> Vec<vk::VertexInputBindingDescription>
    where
        Self: Sized,
    {
        vec![]
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct VertexBlock {
    pub pos: glm::Vec3,
    norm: glm::Vec3,
    uv: glm::Vec2,
    face_index: u32,
}

impl Default for VertexBlock {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            face_index: Default::default(),
            norm: Default::default(),
            uv: Default::default(),
        }
    }
}

impl Vertex for VertexBlock {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default().binding(0).location(0).format(vk::Format::R32G32B32_SFLOAT).offset(0),
            vk::VertexInputAttributeDescription::default().binding(0).location(1).format(vk::Format::R32G32B32_SFLOAT).offset(memoffset::offset_of!(VertexBlock, norm) as u32),
            vk::VertexInputAttributeDescription::default().binding(0).location(2).format(vk::Format::R32G32_SFLOAT).offset(memoffset::offset_of!(VertexBlock, uv) as u32),
            vk::VertexInputAttributeDescription::default().binding(0).location(3).format(vk::Format::R32_UINT).offset(memoffset::offset_of!(VertexBlock, face_index) as u32),
        ]
        .to_vec()
    }
}

impl VertexBlock {
    const VERTEX_MESH_FACES: [VertexBlock; 36] = [
        // right
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 0),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 0),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 0),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 0),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 0),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 0),
        // Left face
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 1),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 1),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 1),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 1),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 1),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 1),
        // Top face
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 1.0), 2),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 1.0), 2),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 0.0), 2),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 0.0), 2),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 0.0), 2),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 1.0), 2),
        // Bottom face
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 1.0), 3),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 0.0), 3),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 1.0), 3),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 0.0), 3),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 1.0), 3),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 0.0), 3),
        // Front face
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 1.0), 5),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 0.0), 5),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 1.0), 5),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 0.0), 5),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 1.0), 5),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 0.0), 5),
        // Back face
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 1.0), 4),
        VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 1.0), 4),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 0.0), 4),
        VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 0.0), 4),
        VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 0.0), 4),
        VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 1.0), 4),
    ];

    pub const fn new(pos: glm::Vec3, norm: Vec3, uv: Vec2, face_index: u32) -> Self {
        Self { pos, norm, uv, face_index }
    }
    pub fn new_quad(pos: glm::Vec3, size: glm::Vec3) -> Vec<VertexBlock> {
        let mut quad_vertices = vec![];

        let x_max = pos.x + size.x;
        let y_max = pos.y + size.y;
        let z_max = pos.z + size.z;

        let front_b_l = Vec3::new(pos.x, pos.y, pos.z);
        let front_b_r = Vec3::new(x_max, pos.y, pos.z);
        let front_t_l = Vec3::new(pos.x, y_max, pos.z);
        let front_t_r = Vec3::new(x_max, y_max, pos.z);

        let back_b_l = Vec3::new(x_max, pos.y, z_max);
        let back_b_r = Vec3::new(pos.x, pos.y, z_max);
        let back_t_l = Vec3::new(x_max, y_max, z_max);
        let back_t_r = Vec3::new(pos.x, y_max, z_max);

        let right_order = vec![
            back_t_l,  //right_top_right,
            front_t_r, //right_top_left,
            front_b_r, //right_bot_left,
            front_b_r, //right_bot_left,
            back_b_l,  // right_bot_right,
            back_t_l,  //right_top_right,
        ];

        let right_uv = vec![Vec2::new(0.0, 0.0), Vec2::new(size.x, 0.0), Vec2::new(size.x, size.y), Vec2::new(size.x, size.y), Vec2::new(0.0, size.y), Vec2::new(0.0, 0.0)];

        let left_order = vec![back_t_r, front_b_l, front_t_l, front_b_l, back_t_r, back_b_r];

        let left_uv = vec![Vec2::new(size.x, 0.0), Vec2::new(0.0, size.y), Vec2::new(0.0, 0.0), Vec2::new(0.0, size.y), Vec2::new(size.x, 0.0), Vec2::new(size.x, size.y)];

        let top_order = vec![front_t_l, front_t_r, back_t_l, back_t_l, back_t_r, front_t_l];

        let top_uv = vec![Vec2::new(0.0, size.z), Vec2::new(size.x, size.z), Vec2::new(size.x, 0.0), Vec2::new(size.x, 0.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, size.z)];

        let bot_order = vec![front_b_l, back_b_l, front_b_r, back_b_l, front_b_l, back_b_r];

        let bot_uv = vec![Vec2::new(0.0, size.z), Vec2::new(size.x, 0.0), Vec2::new(size.x, size.z), Vec2::new(size.x, 0.0), Vec2::new(0.0, size.z), Vec2::new(0.0, 0.0)];

        let front_order = vec![front_b_r, front_t_l, front_b_l, front_t_l, front_b_r, front_t_r];

        let front_uv = vec![Vec2::new(size.x, size.y), Vec2::new(0.0, 0.0), Vec2::new(0.0, size.y), Vec2::new(0.0, 0.0), Vec2::new(size.x, size.y), Vec2::new(size.x, 0.0)];

        let back_order = vec![back_b_l, back_b_r, back_t_r, back_t_r, back_t_l, back_b_l];

        let back_uv = vec![Vec2::new(size.x, size.y), Vec2::new(0.0, size.y), Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(size.x, 0.0), Vec2::new(size.x, size.y)];

        Self::generate_face(&mut quad_vertices, &right_order, &right_uv, &Vec3::new(1.0, 0.0, 0.0), 0);
        Self::generate_face(&mut quad_vertices, &left_order, &left_uv, &Vec3::new(-1.0, 0.0, 0.0), 1);
        Self::generate_face(&mut quad_vertices, &top_order, &top_uv, &Vec3::new(0.0, 1.0, 0.0), 2);
        Self::generate_face(&mut quad_vertices, &bot_order, &bot_uv, &Vec3::new(0.0, -1.0, 0.0), 3);
        Self::generate_face(&mut quad_vertices, &front_order, &front_uv, &Vec3::new(0.0, 0.0, 1.0), 5);
        Self::generate_face(&mut quad_vertices, &back_order, &back_uv, &Vec3::new(0.0, 0.0, -1.0), 4);
        quad_vertices
    }

    fn generate_face(vertices: &mut Vec<VertexBlock>, position: &Vec<Vec3>, uv: &Vec<Vec2>, norm: &Vec3, face: u32) {
        for i in 0..position.len() {
            vertices.push(VertexBlock::new(position[i], *norm, uv[i], face));
        }
    }

    pub fn get_face(face: u32) -> Vec<VertexBlock> {
        let offset = face as usize * 6;

        Self::VERTEX_MESH_FACES[offset..offset + 6].to_vec()
    }

    pub fn get_mesh() -> &'static [VertexBlock; 36] {
        &Self::VERTEX_MESH_FACES
    }
}

#[repr(C)]
pub struct MeshImGui {
    pos: glm::Vec2,
    coords: glm::Vec2,
    color: (u8, u8, u8, u8),
}

impl MeshImGui {
    pub fn create_mesh(draw_data: &imgui::DrawData) -> (Vec<imgui::DrawVert>, Vec<u16>) {
        let mut vertices = Vec::with_capacity(draw_data.total_vtx_count as usize);
        let mut indices = Vec::with_capacity(draw_data.total_idx_count as usize);

        for draw_list in draw_data.draw_lists() {
            vertices.extend_from_slice(draw_list.vtx_buffer());
            indices.extend_from_slice(draw_list.idx_buffer());
        }
        (vertices, indices)
    }
}

impl Vertex for MeshImGui {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default().binding(0).location(0).format(vk::Format::R32G32_SFLOAT).offset(0),
            vk::VertexInputAttributeDescription::default().binding(0).location(1).format(vk::Format::R32G32_SFLOAT).offset(memoffset::offset_of!(MeshImGui, coords) as u32),
            vk::VertexInputAttributeDescription::default().binding(0).location(2).format(vk::Format::R8G8B8A8_UNORM).offset(memoffset::offset_of!(MeshImGui, color) as u32),
        ]
        .to_vec()
    }
}
