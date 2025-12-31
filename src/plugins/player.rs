use crate::components::{key_mapping::*, player::*};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.add_systems(Update, attach_players_controls);
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
    // KeyMapping will be added here once LDTK Player entity is added
}

fn attach_players_controls(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player), (Added<Player>, Without<KeyMapping>)>,
) {
    for (entity, player) in &mut players {
        let key_mapping = match player.player_id {
            0 => KeyMapping::wasd(),
            1 => KeyMapping::arrow_keys(),
            _ => KeyMapping::default(),
        };

        commands.entity(entity).insert(key_mapping);
    }
}
