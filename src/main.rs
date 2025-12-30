use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

mod player;
use player::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(1280.0 / 4.0, 720.0 / 4.0, 0.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tyled.ldtk").into(),
        ..Default::default()
    });
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                move_players_from_input,
                setup_player_controls,
                translate_grid_coords_player_entities,
            ),
        );
        app.insert_resource(LevelSelection::index(0));
        app.register_ldtk_entity::<PlayerBundle>("Player");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins(GamePlugin)
        .run();
}
