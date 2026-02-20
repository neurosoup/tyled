use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// [Component] that stores grid-based coordinate information.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct GridCoords {
    pub x: i32,
    pub y: i32,
}

impl From<IVec2> for GridCoords {
    fn from(i_vec_2: IVec2) -> Self {
        GridCoords {
            x: i_vec_2.x,
            y: i_vec_2.y,
        }
    }
}

impl From<GridCoords> for IVec2 {
    fn from(grid_coords: GridCoords) -> Self {
        IVec2::new(grid_coords.x, grid_coords.y)
    }
}

impl From<TilePos> for GridCoords {
    fn from(tile_pos: TilePos) -> Self {
        GridCoords {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }
}

impl From<GridCoords> for TilePos {
    fn from(grid_coords: GridCoords) -> Self {
        TilePos::new(grid_coords.x as u32, grid_coords.y as u32)
    }
}

impl Add<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn add(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<GridCoords> for GridCoords {
    fn add_assign(&mut self, rhs: GridCoords) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn sub(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<GridCoords> for GridCoords {
    fn sub_assign(&mut self, rhs: GridCoords) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn mul(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<GridCoords> for GridCoords {
    fn mul_assign(&mut self, rhs: GridCoords) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl GridCoords {
    pub fn new(x: i32, y: i32) -> GridCoords {
        GridCoords { x, y }
    }
}
