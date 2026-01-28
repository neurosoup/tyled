use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashSet;

#[derive(Default, Component)]
pub struct Ground;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct GroundCellBundle {
    ground: Ground,
}

#[derive(Default, Resource)]
pub struct LevelLookup {
    pub ground_locations: HashSet<GridCoords>,
    pub level_width: i32,
    pub level_height: i32,
}

impl LevelLookup {
    pub fn on_ground(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x > 0
            && grid_coords.y > 0
            && grid_coords.x <= self.level_width
            && grid_coords.y <= self.level_height
            && self.ground_locations.contains(grid_coords)
    }
}
