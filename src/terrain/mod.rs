use ash::vk::ObjectType;
use block::{BlockType, GPUBlock};
use glm::{Mat4, Vec3};
use libnoise::{Generator, Source};
use octree::Octree;

use crate::vulkan::mesh::{Face, Vertex, VertexBlock};

pub mod block;
pub mod octree;

struct Range {
    start: f32,
    end: f32,
}

// over 0.9 is surface
const SURFACE_LEVEL: Range = Range {
    start: 0.9,
    end: 1.0,
};
const STONE_LEVEL: Range = Range {
    start: 0.9,
    end: 0.0,
};
const BASE_HEIGHT: u32 = 10;

pub enum Biome {
    FlatLand,
    Desert,
    Forest,
    Mountain,
}

pub struct Mesh {
    vertices: Vec<VertexBlock>,
    indices: Vec<u16>,
}

pub struct GreedyMesh;

impl GreedyMesh {
    pub fn create_greedy(mut grid: Vec<GPUBlock>) -> Vec<VertexBlock> {
        let mut quads = vec![];
        for y in 0..1 {
            quads.extend(Self::create_quads(&mut grid, y));
        }
        quads
    }

    fn create_quads(grid: &mut Vec<GPUBlock>, y: usize) -> Vec<VertexBlock> {
        let mut x_start = 0;
        let mut z_start = 0;

        let y_offset = y * CHUNK_AREA_LENGTH;

        // find a non empty place
        'outer: for z in 0..CHUNK_LENGTH {
            let z_offset = z * CHUNK_LENGTH;
            for x in 0..CHUNK_LENGTH {
                let block = grid[x + z_offset + y_offset];

                if block.block_type() != BlockType::Air {
                    x_start = x;
                    z_start = z;
                    break 'outer;
                }
            }
        }

        let mut width = 0;
        let mut length = 0;

        let mut steps_z = 0;
        let mut quads = vec![];

        'outer: loop {
            if (width + x_start + 1) < CHUNK_LENGTH {
                let block = &mut grid[width + y_offset];

                if block.block_type() != BlockType::Air {
                    width += 1;
                    block.texture_index = BlockType::Air;
                }
            } else {
                // check in z direction now

                loop {
                    if (length + z_start + 1) < CHUNK_LENGTH {
                        let z_offset = (length + z_start + 1) * CHUNK_LENGTH;

                        let mut extend = true;
                        for w in 0..width {
                            let new_block = grid[z_offset + w + x_start].block_type();

                            if new_block == BlockType::Air {
                                extend = false;
                                break;
                            }
                        }

                        if extend {
                            length += 1;
                            for w in 0..width {
                                let new_block = &mut grid[z_offset + w + x_start];
                                new_block.texture_index = BlockType::Air;
                            }
                        } else {
                            //

                            // create quad
                            quads.extend(VertexBlock::new_quad(
                                Vec3::new(x_start as f32, y as f32 - 1.0, z_start as f32),
                                Vec3::new(width as f32, 1.0, length as f32),
                            ));

                            x_start = x_start + length;
                            z_start = z_start + 1;
                            width = 0;
                            length = 0;
                            break;
                        }
                    } else {
                        quads.extend(VertexBlock::new_quad(
                            Vec3::new(x_start as f32, y as f32 - 1.0, z_start as f32),
                            Vec3::new(width as f32, 1.0, length as f32),
                        ));

                        break 'outer;
                    }
                }
            }
        }

        quads
    }
}

pub struct World {
    player_pos: Vec3,
    player_distance: usize,

    root: Octree,
}

impl World {
    /// Player distance in chunk range
    pub fn new(player_pos: Vec3, player_distance: usize) -> Self {
        let root = Octree::new(
            glm::Vec2::new(player_pos.x, player_pos.z),
            player_distance,
            5,
        );

        // let chunk_start_x = (player_pos.x as f64 / CHUNK_LENGTH as f64) - 2 as f64;
        // let chunk_start_z = (player_pos.z as f64 / CHUNK_LENGTH as f64) - 2 as f64;

        // let chunk_area_x = (chunk_start_x / CHUNK_AREA_LENGTH as f64).floor() as i32;
        // let chunk_area_z = (chunk_start_z / CHUNK_AREA_LENGTH as f64).floor() as i32;

        // let chunk_distance = 2 * 2;

        // let area_amount = (chunk_distance as f64 / CHUNK_AREA_LENGTH as f64).ceil() as i32;

        // let mut chunk_areas = vec![];

        // chunk_areas.push(ChunkArea::new((chunk_area_x, chunk_area_z)));

        // for z in 0..area_amount {
        //     for x in 0..area_amount {
        //         chunk_areas.push(ChunkArea::new((chunk_area_x + x, chunk_area_z + z)));
        //     }
        // }

        Self {
            player_pos,
            player_distance,
            root,
        }
    }

    pub fn get_culled(&self, player_pos: Vec3) -> Vec<GPUBlock> {
        // let mut objects = vec![];
        // for area in &self.chunk_areas {
        //     objects.extend(area.get_culled_objects().clone());
        // }

        // objects

        // let node = self.root.find_node(glm::Vec2::new(player_pos.x, player_pos.z));
        // node.get_objects()
        todo!();
    }
}

const CHUNK_AREA_LENGTH: usize = 16;

struct ChunkArea {
    chunks: Vec<Chunk>,
    // Area offset
    pub offset: (i32, i32),
}

impl ChunkArea {
    pub fn new(offset: (i32, i32)) -> ChunkArea {
        let chunk_start_x = offset.0 * CHUNK_AREA_LENGTH as i32 - 1;
        let chunk_start_z = offset.1 * CHUNK_AREA_LENGTH as i32 - 1;

        let mut chunks = vec![];
        for z in 0..CHUNK_AREA_LENGTH as i32 + 2 {
            for x in 0..CHUNK_AREA_LENGTH as i32 + 2 {
                chunks.push(Chunk::new(chunk_start_x + x, chunk_start_z + z));
            }
        }

        for z in 1..CHUNK_AREA_LENGTH + 1 {
            let z_offset = z * (CHUNK_AREA_LENGTH + 2);
            for x in 1..CHUNK_AREA_LENGTH + 1 {
                let block_offset = z_offset + x;

                chunks[block_offset].culled_blocks = Chunk::occlusion_cull(
                    &chunks[block_offset].all_blocks,
                    &chunks[block_offset + 1],
                    &chunks[block_offset - 1],
                    &chunks[block_offset + CHUNK_AREA_LENGTH + 2],
                    &chunks[block_offset - CHUNK_AREA_LENGTH + 2],
                );
            }
        }

        Self { chunks, offset }
    }
    pub fn get_culled_objects(&self) -> Vec<VertexBlock> {
        let mut culled = vec![];
        let mut ii = 0;
        for z in 1..CHUNK_AREA_LENGTH + 1 {
            let z_offset = z * (CHUNK_AREA_LENGTH + 2);
            for x in 1..CHUNK_AREA_LENGTH + 1 {
                let block_offset = z_offset + x;
                ii = block_offset;
                culled.extend(self.chunks[block_offset].quads.clone());
            }
        }

        culled
    }
}

/// Save the chunks into a uniform buffer with their model. Then the object will
pub const CHUNK_LENGTH: usize = 64;
pub const VOXEL_SCALE: f32 = 1.0;
const CHUNK_HEIGHT: usize = 90;

pub struct Chunk {
    pub all_blocks: Vec<GPUBlock>,
    pub quads: Vec<VertexBlock>,
    pub culled_blocks: Vec<GPUBlock>,
    pub binary_grid: Vec<u64>,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        let all_blocks = Self::generate_chunk(x, z);
        //  let all_blocks = Self::generate_chunk_test(x, z, BlockType::Air).all_blocks;
        let all_blockss = all_blocks.clone();
        Self {
            all_blocks,
            culled_blocks: vec![],
            quads: GreedyMesh::create_greedy(all_blockss),
            binary_grid: vec![],
        }
    }

    fn update_binary_mask(&mut self) {
        let mut grid: Vec<u64> = Vec::with_capacity(64 * 64);
    }

    pub fn generate_chunk_test(chunk_x: i32, chunk_z: i32, block_type: BlockType) -> Chunk {
        let x_start = chunk_x as f32 * CHUNK_LENGTH as f32;
        let z_start = chunk_z as f32 * CHUNK_LENGTH as f32;

        let mut grid = [[0u32; 16]; 16];
        let amplitude = 10.0;
        let seed = 12004690;
        let hill_effect = 1.0;
        let generator = Source::simplex(seed).add(1.0).scale([0.01, 0.01]);

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                let nx = (x as f64 + x_start as f64) / 16.0;
                let nz = (z as f64 + z_start as f64) / 16.0;

                let noise = generator.sample([nx + x_start as f64, nz + z_start as f64]);
                let processed_noise = (((noise * hill_effect).round() / hill_effect) * amplitude)
                    .round() as u32
                    + (CHUNK_HEIGHT as u32 - amplitude as u32);

                grid[x][z] = processed_noise;
            }
        }

        let mut gpu_blocks = vec![];
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_LENGTH {
                for x in 0..CHUNK_LENGTH {
                    let height = grid[x][z];
                    gpu_blocks.push(GPUBlock::new(
                        Vec3::new(x as f32 + x_start, y as f32, z as f32 + z_start),
                        block_type,
                    ));
                }
            }
        }

        //  Self { all_blocks: gpu_blocks, culled_blocks: vec![], quads: vec![] };
        todo!();
    }

    pub fn generate_chunk(chunk_x: i32, chunk_z: i32) -> Vec<GPUBlock> {
        let x_start = chunk_x as f32 * 16.0;
        let z_start = chunk_z as f32 * 16.0;
        let mut grid = [[0u32; 16]; 16];
        let amplitude = 10.0;
        let seed = 12004690;
        let hill_effect = 15.0;
        let generator = Source::simplex(seed).add(1.0).scale([0.2, 0.2]);

        let surface_start = (SURFACE_LEVEL.start * CHUNK_HEIGHT as f32).round() as u32;

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                let nx = (x as f64 + x_start as f64) / CHUNK_LENGTH as f64;
                let nz = (z as f64 + z_start as f64) / CHUNK_LENGTH as f64;
                grid[x][z] = surface_start;
                // grid[x][z] =
                //     (((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * amplitude).round() as u32 + surface_start;
            }
        }

        let mut gpu_blocks = vec![];

        let surface_end = (SURFACE_LEVEL.end * CHUNK_HEIGHT as f32).round();

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_LENGTH {
                for x in 0..CHUNK_LENGTH {
                    let height = grid[x][z];
                    if height < y as u32 {
                        gpu_blocks.push(GPUBlock::new(
                            Vec3::new(x as f32 + x_start, y as f32, z as f32 + z_start),
                            BlockType::Air,
                        ))
                    } else {
                        if y as u32 >= surface_start {
                            gpu_blocks.push(GPUBlock::new(
                                Vec3::new(x as f32 + x_start, y as f32, z as f32 + z_start),
                                BlockType::Dirt,
                            ));
                        } else {
                            gpu_blocks.push(GPUBlock::new(
                                Vec3::new(x as f32 + x_start, y as f32, z as f32 + z_start),
                                BlockType::Stone,
                            ));
                        }
                    }
                }
            }
        }
        gpu_blocks
    }

    pub fn occlusion_cull(
        objects: &Vec<GPUBlock>,
        right: &Chunk,
        left: &Chunk,
        front: &Chunk,
        back: &Chunk,
    ) -> Vec<GPUBlock> {
        let mut culled_objects = vec![];

        Self::corner_cases(&mut culled_objects, objects, right, left, front, back);
        Self::edges_cases(&mut culled_objects, objects, right, left, front, back);
        Self::middle_cases(&mut culled_objects, objects);

        culled_objects
    }

    pub fn get_objects(&self) -> Vec<GPUBlock> {
        // let mut culled = vec![];

        // for i in 0..self.all_blocks.len() {
        //     if self.all_blocks[i].block_type().bit_mask() != BlockType::Air.bit_mask() {
        //         culled.push(self.all_blocks[i]);
        //     }
        // }
        // self.quads.clone()
        todo!();
    }

    fn get_block_up_bitmask(
        blocks: &Vec<GPUBlock>,
        current_offset: usize,
        current_y: usize,
    ) -> u64 {
        if (current_y + 1) == CHUNK_HEIGHT {
            return 1;
        }
        blocks[current_offset + CHUNK_LENGTH * CHUNK_LENGTH]
            .block_type()
            .bit_mask()
    }

    fn corner_cases(
        culled_objects: &mut Vec<GPUBlock>,
        objects: &Vec<GPUBlock>,
        right: &Chunk,
        left: &Chunk,
        front: &Chunk,
        back: &Chunk,
    ) {
        let mut chunk_area = CHUNK_LENGTH * CHUNK_LENGTH;
        // do the x edge with chunk (Back and front)

        // TODO, check edges
        for y in 0..CHUNK_HEIGHT {
            let y_offset = y * chunk_area;
            let y_offset_down = (y + 1) * chunk_area;

            let offset_top_right = y_offset + (CHUNK_LENGTH * CHUNK_LENGTH) - 1;
            let offset_top_left = y_offset + (CHUNK_LENGTH * (CHUNK_LENGTH - 1));
            let offset_bot_right = y_offset + (CHUNK_LENGTH) - 1;
            let offset_bot_left = y_offset;

            // Bottom Left Corner
            {
                let bot_left = objects[offset_bot_left];

                if bot_left.block_type() != BlockType::Air {
                    let block_behind = back.all_blocks[offset_top_left].block_type().bit_mask();
                    let block_left = left.all_blocks[offset_bot_right].block_type().bit_mask();

                    let block_front = objects[offset_bot_left + CHUNK_LENGTH]
                        .block_type()
                        .bit_mask();
                    let block_right = objects[offset_bot_left + 1].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, offset_bot_left, y);

                    let is_air = (block_behind | block_left | block_front | block_right | block_up)
                        & BlockType::Air.bit_mask();

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(bot_left);
                    }
                }
            }

            // Bottom Right Corner
            {
                let bot_right = objects[offset_bot_right];

                if bot_right.block_type() != BlockType::Air {
                    let block_behind = back.all_blocks[offset_top_right].block_type().bit_mask();
                    let block_right = right.all_blocks[offset_bot_left].block_type().bit_mask();

                    let block_front = objects[offset_bot_right + CHUNK_LENGTH]
                        .block_type()
                        .bit_mask();
                    let block_left = objects[offset_bot_right - 1].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, offset_bot_right, y);

                    let is_air =
                        (block_behind | block_right | block_front | block_left | block_up) & 1;

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(bot_right);
                    }
                }
            }

            // Top Left Corner
            {
                let bot_left = objects[offset_top_left];

                if bot_left.block_type() != BlockType::Air {
                    let block_front = front.all_blocks[offset_bot_left].block_type().bit_mask();
                    let block_left = left.all_blocks[offset_top_right].block_type().bit_mask();

                    let block_behind = objects[offset_top_left - CHUNK_LENGTH]
                        .block_type()
                        .bit_mask();
                    let block_right = objects[offset_top_left + 1].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, offset_top_left, y);

                    let is_air =
                        (block_behind | block_right | block_front | block_left | block_up) & 1;

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(bot_left);
                    }
                }
            }

            // Top Right Corner
            {
                let bot_left = objects[offset_top_right];

                if bot_left.block_type() != BlockType::Air {
                    let block_front = front.all_blocks[offset_bot_right].block_type().bit_mask();
                    let block_right = right.all_blocks[offset_top_left].block_type().bit_mask();

                    let block_behind = objects[offset_top_right - CHUNK_LENGTH]
                        .block_type()
                        .bit_mask();
                    let block_left = objects[offset_top_right - 1].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, offset_top_right, y);

                    let is_air = (block_behind | block_right | block_front | block_left | block_up)
                        & BlockType::Air.bit_mask();

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(bot_left);
                    }
                }
            }
        }
    }

    fn edges_cases(
        culled_objects: &mut Vec<GPUBlock>,
        objects: &Vec<GPUBlock>,
        right: &Chunk,
        left: &Chunk,
        front: &Chunk,
        back: &Chunk,
    ) {
        let x_offset = 0;
        let z_offset = CHUNK_LENGTH * 1;
        let mut chunk_area = CHUNK_LENGTH * CHUNK_LENGTH;
        // do the x edge with chunk (Back and front)

        // TODO, check edges

        // CHECK FRONT AND BACK EDGES
        for y in 0..CHUNK_HEIGHT {
            let y_offset = y * chunk_area;
            for x in 1..CHUNK_LENGTH - 1 {
                // Bottom Edge

                let lower_object = objects[y_offset + x];

                if lower_object.block_type() != BlockType::Air {
                    let back = back.all_blocks[y_offset + x].block_type().bit_mask();
                    let front = objects[y_offset + CHUNK_LENGTH].block_type().bit_mask();
                    let right = objects[y_offset + x + 1].block_type().bit_mask();
                    let left = objects[y_offset + x - 1].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, y_offset + x, y);

                    let is_air =
                        (back | front | right | left | block_up) & BlockType::Air.bit_mask();

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(lower_object);
                    }
                }
                let top_offset = y_offset + x + CHUNK_LENGTH * (CHUNK_LENGTH - 1);

                let front_object = objects[top_offset];

                if front_object.block_type() != BlockType::Air {
                    let front = front.all_blocks[y_offset + x].block_type().bit_mask();
                    let right = objects[top_offset + 1].block_type().bit_mask();
                    let left = objects[top_offset - 1].block_type().bit_mask();
                    let back = objects[top_offset - 1 - CHUNK_LENGTH]
                        .block_type()
                        .bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, top_offset, y);

                    let is_air =
                        (front | right | left | back | block_up) & BlockType::Air.bit_mask();

                    if is_air == BlockType::Air.bit_mask() {
                        culled_objects.push(front_object);
                    }
                }

                let left_offset = y_offset + (x * CHUNK_LENGTH);
                let left_object = objects[left_offset];
                let right_offset = y_offset + ((CHUNK_LENGTH) * (x + 1)) - 1;

                if left_object.block_type() != BlockType::Air {
                    let front = objects[left_offset + CHUNK_LENGTH].block_type().bit_mask();
                    let back = objects[left_offset - CHUNK_LENGTH].block_type().bit_mask();
                    let right = objects[left_offset + 1].block_type().bit_mask();
                    let left = left.all_blocks[right_offset].block_type().bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, left_offset, y);

                    let air = (front | back | right | left | block_up) & BlockType::Air.bit_mask();

                    if air == BlockType::Air.bit_mask() {
                        culled_objects.push(left_object);
                    }
                }

                let right_object = objects[right_offset];

                if right_object.block_type() != BlockType::Air {
                    let back = objects[y_offset].block_type().bit_mask();
                    let front = objects[right_offset + CHUNK_LENGTH].block_type().bit_mask();
                    let left = objects[right_offset + 1].block_type().bit_mask();
                    let right = right.all_blocks[y_offset + CHUNK_LENGTH * x]
                        .block_type()
                        .bit_mask();

                    let block_up = Self::get_block_up_bitmask(objects, right_offset, y);

                    let air = (front | back | right | left | block_up) & BlockType::Air.bit_mask();

                    if air == BlockType::Air.bit_mask() {
                        culled_objects.push(right_object);
                    }
                }
            }
        }
    }

    fn middle_cases(culled_objects: &mut Vec<GPUBlock>, objects: &Vec<GPUBlock>) {
        for y in 0..CHUNK_HEIGHT {
            let y_offset = y * CHUNK_LENGTH * CHUNK_LENGTH;
            for z in 1..CHUNK_LENGTH - 1 {
                let z_offset = CHUNK_LENGTH * z;
                for x in 1..CHUNK_LENGTH - 1 {
                    let x_offset = x;

                    let block = objects[y_offset + x_offset + z_offset];
                    if block.block_type() != BlockType::Air {
                        if y == CHUNK_HEIGHT {
                            culled_objects.push(block);
                            continue;
                        }

                        let front = objects[y_offset + x_offset + z_offset + CHUNK_LENGTH]
                            .block_type()
                            .bit_mask();
                        let back = objects[y_offset + x_offset + z_offset - CHUNK_LENGTH]
                            .block_type()
                            .bit_mask();
                        let right = objects[y_offset + x_offset + z_offset + 1]
                            .block_type()
                            .bit_mask();
                        let left = objects[y_offset + x_offset + z_offset - 1]
                            .block_type()
                            .bit_mask();

                        let up =
                            Self::get_block_up_bitmask(objects, y_offset + x_offset + z_offset, y);

                        let is_air = (front | back | right | left | up) & 1;
                        if is_air == BlockType::Air.bit_mask() {
                            culled_objects.push(block);
                        }
                    }
                }
            }
        }
    }

    pub fn generate_face(&self, x: usize, y: usize, z: usize, face: usize) {
        // let mut face_vertices = Vec::new();
        // let mut face_indices = Vec::new();

        // VertexBlock::get_face(face as u32);
    }

    pub fn box_blur(mut grid: [[u32; 16]; 16]) -> [[u32; 16]; 16] {
        // TODO, I can optimize this by having two arrays
        // and depending on the boolean, I access only zeroes or access 1 and the value for that grid. so no comparing needed.
        for x in 0..16 {
            for y in 0..16 {
                let mut total_value = 0;
                let mut divide_by = 0;

                total_value += grid[x][y];
                divide_by += 1;

                if x > 0 {
                    // left
                    total_value += grid[x - 1][y];
                    divide_by += 1;
                    if y > 0 {
                        // bottom left and bottom
                        total_value += grid[x - 1][y - 1];
                        total_value += grid[x][y - 1];
                        divide_by += 2;

                        if x < 15 {
                            // bottom right
                            total_value += grid[x + 1][y - 1];
                            divide_by += 1;
                        }
                    }
                }
                if x < 15 {
                    // right
                    total_value += grid[x + 1][y];
                    divide_by += 1;

                    if y < 15 {
                        // top and top right
                        total_value += grid[x + 1][y + 1];
                        total_value += grid[x][y + 1];
                        divide_by += 2;

                        if x > 0 {
                            // Top left
                            total_value += grid[x - 1][y + 1];
                            divide_by += 1;
                        }
                    }
                }

                grid[x][y] = total_value / divide_by;
            }
        }
        grid
    }
}

pub struct SimplexNoise {}
impl SimplexNoise {
    const PERM: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];

    fn hash(i: u32) -> u8 {
        Self::PERM[i as u8 as usize]
    }
    fn grad(hash: u32, x: f32) -> f32 {
        let h = hash & 0x0F;
        let mut grad = 1.0 + (h & 7) as f32;

        if (h & 8) != 0 {
            grad *= -1.0;
        }

        grad * x
    }

    fn grad_2d(hash: u32, x: f32, y: f32) -> f32 {
        let h = hash & 0x3F;
        let (mut u, v) = {
            if h < 4 {
                (y, x)
            } else {
                (x, y)
            }
        };

        if h & 1 == 1 {
            u *= -1.0;
        }

        let mut v_multi = 2.0;

        if h & 2 == 2 {
            v_multi = -2.0;
        }

        u + v * v_multi
    }

    pub fn noise_2d(
        x: usize,
        y: usize,
        frequency: f32,
        seed: u32,
        amplitude: f32,
        persistence: f32,
        octaves_count: u32,
    ) -> f32 {
        let mut noise_value = 0.0;

        let mut inner_amplitude = 1.0;
        let mut total_amplitude = 0.0;

        let mut inner_freq = 1.0;
        let octal_offset_x = 5.4;
        let octal_offset_y = 2.4;

        let hill_steps = 5.0;

        for i in 0..octaves_count {
            noise_value += Self::two_d(
                x as f32 * frequency * inner_freq * (octal_offset_x * i as f32) + seed as f32,
                y as f32 * frequency * inner_freq * (octal_offset_y * i as f32) + seed as f32,
            ) * inner_amplitude;

            total_amplitude += inner_amplitude;

            inner_amplitude *= persistence;
            inner_freq *= 2.0;
        }
        noise_value /= total_amplitude;
        //  noise_value = noise_value.powf(0.31);
        noise_value = (noise_value + 1.0) / 2.0;
        noise_value = (noise_value * hill_steps).round() / hill_steps;
        noise_value *= amplitude;
        noise_value
    }

    pub fn generate_noise(x: f32, y: f32, octaves: u32, persistence: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += Self::two_d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        (total / max_value + 1.0) * 0.5
    }

    pub fn one_d(x: f32) -> f32 {
        let (n0, n1);

        // Corners coordinates (nearest integer values):
        let i0 = x.floor();
        let i1 = i0 + 1.0;
        // Distances to corners (between 0 and 1):
        let x0 = x - i0;
        let x1 = x0 - 1.0;

        let mut t0 = 1.0 - x0 * x0;
        t0 *= t0;
        n0 = t0 * t0 * Self::grad(Self::PERM[i0 as usize] as u32, x0);

        let mut t1 = 1.0 - x1 * x1;
        t1 *= t1;
        n1 = t1 * t1 * Self::grad(Self::PERM[i1 as usize] as u32, x1);

        0.395 * (n0 + n1)
    }

    pub fn two_d(x: f32, y: f32) -> f32 {
        const F2: f32 = 0.366025403;
        const G2: f32 = 0.211324865;

        // Skew the input space to determine which simplex cell we're in
        let s: f32 = (x + y) * F2;
        let xs: f32 = x + s;
        let ys: f32 = y + s;
        let i = xs.floor();
        let j = ys.floor();

        // Unskew the cell origin back to (x,y) space
        let t = (i + j) as f32 * G2;
        let _x0 = i - t;
        let _y0 = j - t;
        let x0 = x - _x0;
        let y0 = y - _y0;

        // For the 2D case, the simplex shape is an equilateral triangle.
        // Determine which simplex we are in.
        let (mut i1, mut j1) = (0, -1);

        if x0 > y0 {
            i1 = 1;
            j1 = 0;
        }

        let x1 = x0 - (i1 as f32) + G2;
        let y1 = y0 - (j1 as f32) + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let gi0_hash = i as u32 + Self::hash(j as u32) as u32;
        let gi1_hash = (i + i1 as f32) as u32 + Self::hash((j + j1 as f32) as u32) as u32;
        let gi2_hash = (i + 1.0) as u32 + Self::hash((j + 1.0) as u32) as u32;

        let gi0 = Self::hash(gi0_hash as u32);
        let gi1 = Self::hash(gi1_hash as u32);
        let gi2 = Self::hash(gi2_hash as u32);

        let (n0, n1, n2);

        // Calculate the contribution
        let mut t0 = 0.5 - x0 * x0 - y0 * y0;

        if t0 < 0.0 {
            n0 = 0.0;
        } else {
            t0 *= t0;
            n0 = t0 * t0 * Self::grad_2d(gi0 as u32, x0, y0);
        }
        // Calculate the contribution
        let mut t1 = 0.5 - x1 * x1 - y1 * y1;

        if t1 < 0.0 {
            n1 = 0.0;
        } else {
            t1 *= t1;
            n1 = t1 * t1 * Self::grad_2d(gi1 as u32, x1, y1);
        }
        // Calculate the contribution
        let mut t2 = 0.5 - x2 * x2 - y2 * y2;

        if t2 < 0.0 {
            n2 = 0.0;
        } else {
            t2 *= t2;
            n2 = t2 * t2 * Self::grad_2d(gi2 as u32, x2, y2);
        }

        45.23065 * (n0 + n1 + n2)
    }
}
