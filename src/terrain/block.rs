use glm::{Vec3, Vec4};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C, align(4))]
pub enum BlockType {
    Air,
    Dirt,
    Grass,
    Stone,
    AcaciaL,
    Sand,
}
impl BlockType {
    pub const fn variants() -> usize {
        6
    }
    pub fn as_raw(&self) -> u32 {
        *self as u32
    }

    pub fn bit_mask(&self) -> u64 {
        let value = self.as_raw();

        1 << value
    }
}
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct GPUBlock {
    // Center of object
    pub position: Vec3,
    /// Which chunk it belongs to
    pub texture_index: BlockType,

    pub scale: Vec3,
}

impl GPUBlock {
    pub fn new(position: Vec3, block_type: BlockType) -> Self {
        Self { position, texture_index: block_type, scale: Vec3::one() }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self { position, texture_index: BlockType::Air, scale: Vec3::one() }
    }

    pub fn block_type(&self) -> BlockType {
        self.texture_index
    }
}
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct GPUTexture {
    ambient: Vec3,
    shininess: f32,
    diffuse: Vec4,
    specular: Vec3,
    face_indices: [u32; 6],
}

impl GPUTexture {
    pub fn new(ambient: Vec3, diffuse: Vec3, specular: Vec3, face_indices: [u32; 6]) -> Self {
        let d = Vec4::from(diffuse);

        Self { face_indices, ambient, shininess: 0.0, diffuse: d, specular }
    }

    pub fn from_face_indices(face_indices: [u32; 6]) -> Self {
        Self { face_indices, ..Default::default() }
    }
}

impl Default for GPUTexture {
    fn default() -> Self {
        let n_ambient = Vec3::new(0.1, 0.1, 0.1);
        let n_diffuse = Vec3::new(0.5, 0.5, 0.5);
        let specular = Vec3::new(0.4, 0.4, 0.4);

        Self {
            face_indices: Default::default(),
            ambient: n_ambient,
            shininess: 0.0,
            diffuse: Vec4::from(n_diffuse),
            specular,
        }
    }
}

pub struct Materials {}

impl Materials {
    pub fn get_all() -> Vec<GPUTexture> {
        let atlas_width = 29;

        let mut gpu_textures = [GPUTexture::default(); BlockType::variants()];
        let dirt_index = 8 * atlas_width + 16;

        let grass_side_index = 10 * atlas_width + 16;
        let grass_top_index = 14 * atlas_width + 16 + 1;
        let sand_index = 11 * atlas_width + 15;

        let stone_index = 12;

        let acacia_top = 2;
        let acacia_side = 1;

        // right-> left -> top -> bot -> front -> back

        let dirt_block = [dirt_index; 6];

        let mut grass_block = [grass_side_index; 6];
        grass_block[2] = grass_top_index;
        grass_block[3] = dirt_index;

        let stone_block = [stone_index; 6];

        let mut acacia_block = [acacia_side; 6];
        acacia_block[2] = acacia_top;
        acacia_block[3] = acacia_top;

        let sand_block = [sand_index; 6];

        gpu_textures[BlockType::Dirt as usize] = GPUTexture::from_face_indices(dirt_block);
        gpu_textures[BlockType::Grass as usize] = GPUTexture::from_face_indices(grass_block);
        gpu_textures[BlockType::Stone as usize] = GPUTexture::from_face_indices(stone_block);
        gpu_textures[BlockType::AcaciaL as usize] = GPUTexture::from_face_indices(acacia_block);
        gpu_textures[BlockType::Sand as usize] = GPUTexture::from_face_indices(sand_block);

        gpu_textures.to_vec()
    }
}
