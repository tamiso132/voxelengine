use std::ops::Div;

use crate::terrain::Chunk;
use glm::{Vec2, Vec3};

use super::{block::GPUBlock, CHUNK_LENGTH, VOXEL_SCALE};

// lazily allocate them
type Chunkindex = u32;

struct Range {
    pub start: usize,
    pub end: usize,
}
impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Own the data, it has.
struct Freelist<T> {
    list: Vec<T>,
    free: Vec<Range>,
}

impl<T> Freelist<T> {
    pub fn new() -> Freelist<T> {
        Freelist { list: vec![], free: vec![] }
    }

    pub fn add(&mut self, data: T) -> usize {
        let data_index;
        if self.free.len() == 0 {
            self.list.push(data);
            data_index = self.list.len() - 1;
        } else {
            let last_index = self.free.len() - 1;

            data_index = self.free[last_index].end;
            self.free[last_index].end -= 1;

            if self.free[last_index].start == self.free[1].end {
                self.free.remove(last_index);
            }
        }
        data_index
    }
    // might not do a Range, if I dont have a lot of free spots, more memory intensive but O(1) remove.

    pub fn remove(&mut self, index: usize) {
        if self.free.len() == 0 {
            self.free.push(Range::new(index - 1, index));
        } else {
            let mut added = false;
            for e in &mut self.free {
                if (e.end + 1) == index {
                    e.end += 1;
                    added = true;
                    break;
                } else if e.start == index {
                    e.start -= 1;
                    added = true;
                    break;
                }
            }

            if added == false {
                self.free.push(Range::new(index - 1, index));
            }
        }
    }
}

struct Node {
    /// Center position of the node
    pos: glm::Vec2,

    /// Size in voxels
    size: glm::Vec2,

    // Quadrant children
    children: Option<Box<[Node; 4]>>,
    /// indices to global chunk array freelist
    chunks: Vec<Chunk>,

    // does not own it, might only need &, will see
    parent: *mut Node,
}

impl Node {
    pub fn new(parent: *mut Node, position: glm::Vec2, size: glm::Vec2) -> Self {
        Node { pos: position, size, children: None, chunks: vec![], parent }
    }

    pub fn load_player_nodes(&mut self, max_depth: u32, player_pos: Vec2, player_distance: usize) {
        if max_depth > 0 {
            self.split();

            for child in self.children.as_mut().unwrap().iter_mut() {
                if child.distance(player_pos) < player_distance as f32 {
                    child.load_player_nodes(max_depth - 1, player_pos, player_distance);
                }
            }
        } else {
            self.load_chunks();
        }
    }

    pub fn find_node(&self, max_depth: u32, pos: Vec2) -> &Node {
        if max_depth > 0 {
            if self.children.is_some() {
                let children = unsafe { &*self.children.as_ref().unwrap_unchecked() };
                for child in children.iter() {
                    if child.is_inside(pos) {
                        let node = child.find_node(max_depth - 1, pos);
                        return node;
                    }
                }
                panic!("")
            } else {
                return self;
            }
        } else {
            return self;
        }
    }

    pub fn get_objects(&self) -> Vec<GPUBlock> {
        let mut objects = vec![];
        for chunk in &self.chunks {
            objects.extend(chunk.all_blocks.clone());
        }
        objects
    }

    /// the chunks are loaded from bot left -> top right in lines.
    pub fn load_chunks(&mut self) {
        let pos = self.pos;

        let chunks_fit = self.size.x / CHUNK_LENGTH as f32;

        assert!(
            chunks_fit.fract() == 0.0,
            "should not be a fractional chunk. only whole numbers, check Octree max size and depth"
        );

        let pos_offset = (pos - self.size.div(2.0)).div(CHUNK_LENGTH as f32);

        for z in 0..chunks_fit as usize {
            let z_offset = z as f32 * chunks_fit + pos_offset.y;
            for x in 0..chunks_fit as usize {
                let x_offset = pos_offset.y + x as f32;

                self.chunks.push(Chunk::new(x_offset as i32, z_offset as i32));
            }
        }
    }

    pub fn unload_chunk(&mut self) {
        self.chunks.clear();
        // TODO, saving system etc. we will see how we do it
    }

    fn distance(&self, point: Vec2) -> f32 {
        (self.pos.x - point.x).powi(2) + (self.pos.y - point.y).powi(2) - self.size.x / 2.0
    }

    fn is_inside(&self, pos: Vec2) -> bool {
        let half_size_x = self.size.x / 2.0;
        let half_size_z = self.size.y / 2.0;

        if pos.x > (self.pos.x + half_size_x)
            || pos.x < (self.pos.x - half_size_x)
            || pos.y > (self.pos.y + half_size_z)
            || pos.y < (self.pos.y - half_size_z)
        {
            return false;
        }
        true
    }

    pub fn split(&mut self) {
        let s_child = self.size.div(2.0);
        let s_half_child = self.size.div(4.0);

        assert!(s_child.x < CHUNK_LENGTH as f32);
        assert!(s_child.x < CHUNK_LENGTH as f32);
        let p_t_l = Vec2::new(self.pos.x - s_half_child.x, self.pos.y + s_half_child.y);
        let p_t_r = Vec2::new(self.pos.x + s_half_child.x, self.pos.y + s_half_child.y);

        let p_b_l = Vec2::new(self.pos.x - s_half_child.x, self.pos.y - s_half_child.y);
        let p_b_r = Vec2::new(self.pos.x + s_half_child.x, self.pos.y - s_half_child.y);

        // Top left
        let top_left = Node::new(self, p_t_l, s_child);
        // top_right
        let top_right = Node::new(self, p_t_r, s_child);
        // bot_left
        let bot_left = Node::new(self, p_b_l, s_child);
        // bot_right
        let bot_right = Node::new(self, p_b_r, s_child);

        self.children = Some(Box::new([top_left, top_right, bot_left, bot_right]));

        // TODO, if chunks are loaded, should give them to the children then clear.
    }
}
/// Root octree
pub struct Octree {
    root: Node,
    max_depth: u32,
}

impl Octree {
    /// Creates a new world.
    ///
    /// # Arguments
    ///
    /// * `target_pos` - position of the thing that looks. In order to lazily allocate further away chunks.
    /// * `player_view` - How far the target can see in chunks
    /// * `max_depth` - how deep it goes.
    pub fn new(target_pos: Vec2, player_view: usize, max_depth: u32) -> Self {
        let chunk_amount = 2u32.pow(max_depth - 1);
        let player_max_distance = player_view * CHUNK_LENGTH;

        let size_of_world = (chunk_amount as usize * CHUNK_LENGTH) as f32 * VOXEL_SCALE;

        let mut root = Node::new(std::ptr::null_mut(), Vec2::new(0.0, 0.0), Vec2::new(size_of_world, size_of_world));
        root.load_player_nodes(max_depth, target_pos, player_max_distance);

        Self { root, max_depth }
    }

    pub fn find_node(&self, pos: Vec2) -> &Node {
        self.root.find_node(self.max_depth, pos)
    }
}
