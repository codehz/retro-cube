use std::{
    ops::{Index, IndexMut},
    usize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SolidBlock(u8, u8, u8);

impl Into<[f32; 3]> for &SolidBlock {
    fn into(self) -> [f32; 3] {
        [
            self.0 as f32 / 255.0,
            self.1 as f32 / 255.0,
            self.2 as f32 / 255.0,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Empty,
    Solid(SolidBlock),
}

impl Default for Block {
    #[inline(always)]
    fn default() -> Self {
        Self::Empty
    }
}

impl Block {
    #[inline(always)]
    pub fn solid(r: u8, g: u8, b: u8) -> Self {
        Self::Solid(SolidBlock(r, g, b))
    }

    #[inline(always)]
    pub fn from_color(color: u32) -> Self {
        let (b, g, r) = (color >> 16u32 & 0xFF, color >> 8u32 & 0xFF, color & 0xFF);
        Self::solid(r as u8, g as u8, b as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldDimension(pub u32, pub u32, pub u32);

impl WorldDimension {
    pub fn idx(&self, index: WorldPosition) -> usize {
        (index.0 as usize)
            + (self.1 as usize) * ((index.1 as usize) + (self.2 as usize) * (index.2 as usize))
    }

    pub fn pos(&self, index: usize) -> WorldPosition {
        let WorldDimension(_, width, height) = *self;
        let (width, height) = (width as usize, height as usize);
        WorldPosition(
            (index % width) as u32,
            (index / width % height) as u32,
            (index / width / height) as u32,
        )
    }

    pub fn max_faces(&self) -> usize {
        let WorldDimension(x, y, z) = *self;
        let (x, y, z) = (x as usize, y as usize, z as usize);
        let count = x * y * z;
        count * 3 + x * y + x * z + y * z
    }
}

impl From<(u32, u32, u32)> for WorldDimension {
    fn from((x, y, z): (u32, u32, u32)) -> Self {
        Self(x, y, z)
    }
}

#[derive(Debug)]
pub struct World {
    data: Vec<Block>,
    dims: WorldDimension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPosition(pub u32, pub u32, pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl From<(u32, u32, u32)> for WorldPosition {
    fn from((x, y, z): (u32, u32, u32)) -> Self {
        Self(x, y, z)
    }
}

impl Into<[f32; 3]> for WorldPosition {
    fn into(self) -> [f32; 3] {
        [self.0 as f32, self.1 as f32, self.2 as f32]
    }
}

impl Into<u32> for Direction {
    fn into(self) -> u32 {
        match self {
            Direction::North => 0u32,
            Direction::South => 1u32,
            Direction::East => 2u32,
            Direction::West => 3u32,
            Direction::Up => 4u32,
            Direction::Down => 5u32,
        }
    }
}

impl From<u32> for Direction {
    fn from(val: u32) -> Self {
        match val {
            0u32 => Direction::North,
            1u32 => Direction::South,
            2u32 => Direction::East,
            3u32 => Direction::West,
            4u32 => Direction::Up,
            5u32 => Direction::Down,
            _ => panic!("unexpected value"),
        }
    }
}

impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        (0..6).map(From::<u32>::from)
    }

    pub fn apply(
        self,
        WorldDimension(mx, my, mz): WorldDimension,
        WorldPosition(x, y, z): WorldPosition,
    ) -> Option<WorldPosition> {
        match self {
            Direction::North => {
                if z == 0 {
                    None
                } else {
                    Some(WorldPosition(x, y, z - 1))
                }
            }
            Direction::South => {
                if z == mz - 1 {
                    None
                } else {
                    Some(WorldPosition(x, y, z + 1))
                }
            }
            Direction::East => {
                if x == mx - 1 {
                    None
                } else {
                    Some(WorldPosition(x + 1, y, z))
                }
            }
            Direction::West => {
                if x == 0 {
                    None
                } else {
                    Some(WorldPosition(x - 1, y, z))
                }
            }
            Direction::Up => {
                if y == my - 1 {
                    None
                } else {
                    Some(WorldPosition(x, y + 1, z))
                }
            }
            Direction::Down => {
                if y == 0 {
                    None
                } else {
                    Some(WorldPosition(x, y - 1, z))
                }
            }
        }
    }
}

impl Index<WorldPosition> for World {
    type Output = Block;

    #[inline(always)]
    fn index(&self, index: WorldPosition) -> &Self::Output {
        &self.data[self.dims.idx(index)]
    }
}

impl IndexMut<WorldPosition> for World {
    #[inline(always)]
    fn index_mut(&mut self, index: WorldPosition) -> &mut Self::Output {
        let idx = self.dims.idx(index);
        &mut self.data[idx]
    }
}

impl World {
    pub fn dims(&self) -> WorldDimension {
        self.dims
    }

    #[inline(always)]
    pub fn test(&self, index: WorldPosition) -> bool {
        self[index] != Block::Empty
    }

    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = (WorldPosition, &'a SolidBlock)> {
        let dims = self.dims;
        self.data
            .iter()
            .enumerate()
            .filter_map(move |(i, blk)| match blk {
                Block::Empty => None,
                Block::Solid(blk) => Some((dims.pos(i), blk)),
            })
    }

    pub fn new<DIMS: Into<WorldDimension>>(dims: DIMS) -> Self {
        let dims = Into::<WorldDimension>::into(dims);
        let size = (dims.0 as usize) * (dims.1 as usize) * (dims.2 as usize);
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, Default::default);
        Self { data, dims }
    }

    pub fn from_vox(data: &[u8]) -> Self {
        let data = dot_vox::load_bytes(data).unwrap();
        let model = &data.models[0];
        let dot_vox::Size { x, y, z } = model.size;
        let mut res = Self::new((x, z, y));

        for voxel in &model.voxels {
            let dot_vox::Voxel { x, y, z, i } = *voxel;
            let pos = WorldPosition(x as u32, z as u32, y as u32);
            let color = data.palette[i as usize];
            let blk = Block::from_color(color);
            res[pos] = blk;
        }

        res
    }
}
