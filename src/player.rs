use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Default, Component, Debug)]
pub struct Player {
    pub player_id: i32,
    // Note: KeyMapping will be added manually after spawning since it's not in LDTK
}

impl From<&EntityInstance> for Player {
    fn from(entity_instance: &EntityInstance) -> Self {
        Self {
            player_id: *entity_instance
                .get_int_field("player_id")
                .unwrap_or_else(|e| panic!("Failed to get player_id: {}", e)),
        }
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
}

#[derive(Component)]
pub struct KeyMapping {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
}

impl KeyMapping {
    pub fn wasd() -> Self {
        Self {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
        }
    }

    pub fn arrow_keys() -> Self {
        Self {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
        }
    }
}

pub fn move_players_from_input(
    mut players: Query<(&mut GridCoords, &KeyMapping), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut player_grid_coords, key_mapping) in &mut players {
        let movement_direction = if input.just_pressed(key_mapping.up) {
            GridCoords::new(0, 1)
        } else if input.just_pressed(key_mapping.down) {
            GridCoords::new(0, -1)
        } else if input.just_pressed(key_mapping.left) {
            GridCoords::new(-1, 0)
        } else if input.just_pressed(key_mapping.right) {
            GridCoords::new(1, 0)
        } else {
            continue; // No input for this player, continue to next
        };

        let destination = *player_grid_coords + movement_direction;
        *player_grid_coords = destination;
    }
}

const GRID_SIZE: i32 = 16;

pub fn translate_grid_coords_player_entities(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in &mut grid_coords_entities {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}

// System to add key mappings to players after they're spawned from LDTK
pub fn setup_player_controls(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player), (Added<Player>, Without<KeyMapping>)>,
) {
    for (entity, player) in &mut players {
        let key_mapping = match player.player_id {
            0 => KeyMapping::wasd(),
            1 => KeyMapping::arrow_keys(),
            _ => KeyMapping::wasd(), // Default fallback
        };

        commands.entity(entity).insert(key_mapping);
        println!("Added controls for player {:?}", player);
    }
}
