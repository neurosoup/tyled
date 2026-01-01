use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashSet;

#[derive(Default, Component)]
pub struct Walkable;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct WalkableBundle {
    wall: Walkable,
}

#[derive(Default, Resource)]
pub struct LevelWalkables {
    pub walkable_locations: HashSet<GridCoords>,
    pub level_width: i32,
    pub level_height: i32,
}

impl LevelWalkables {
    pub fn in_walkable(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x > 0
            && grid_coords.y > 0
            && grid_coords.x <= self.level_width
            && grid_coords.y <= self.level_height
            && self.walkable_locations.contains(grid_coords)
    }
}
