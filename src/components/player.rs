use crate::prelude::actions::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Default, Component)]
pub struct Player {
    pub player_id: i32,
}

impl From<&EntityInstance> for Player {
    fn from(entity_instance: &EntityInstance) -> Self {
        let player_id = *entity_instance
            .get_int_field("player_id")
            .unwrap_or_else(|e| panic!("Failed to get player_id: {}", e));
        Self { player_id }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[from_entity_instance]
    player: Player,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(initial_direction_lock_state)]
    direction_lock_state: DirectionLockState,
}

fn initial_direction_lock_state(entity_instance: &EntityInstance) -> DirectionLockState {
    let player_id = *entity_instance
        .get_int_field("player_id")
        .unwrap_or_else(|e| panic!("Failed to get player_id: {}", e));
    DirectionLockState::new(match player_id {
        0 => LockedDirection::Down,
        1 => LockedDirection::Up,
        _ => LockedDirection::Down,
    })
}
