use crate::prelude::player::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);
    app.insert_resource(LevelSelection::index(0));
    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.add_systems(Startup, load_ltdk_world);
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
}

fn load_ltdk_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tyled.ldtk").into(),
        ..Default::default()
    });
}
