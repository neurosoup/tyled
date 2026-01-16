use crate::prelude::actions::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Default, Component)]
pub struct DirectionLockCooldown {
    pub timer: Timer,
}

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
    #[with(initial_direction_lock_cooldown)]
    direction_lock_cooldown: DirectionLockCooldown,
}

fn initial_direction_lock_cooldown(_: &EntityInstance) -> DirectionLockCooldown {
    DirectionLockCooldown {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    }
}
