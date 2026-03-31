use crate::prelude::*;
use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    camera::{Viewport, visibility::RenderLayers},
    post_process::dof::DepthOfField,
    prelude::*,
    window::WindowResized,
};
use bevy_ecs_tiled::prelude::{MapCreated, TiledEvent, TiledImage, TiledObject};
use rand::Rng;

pub const HUD_RENDER_LAYER: usize = 1;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 0.80;
/// Minimum zoom scale (zoomed out)
const MIN_ZOOM_SCALE: f32 = 0.33;
/// Maximum zoom scale (zoomed in)
const MAX_ZOOM_SCALE: f32 = 2.0;
/// Base distance for zoom calculations
const BASE_ZOOM_DISTANCE: f32 = 200.0;

// HSL equivalent of srgb_u8(100, 122, 64): saturation and lightness are fixed,
// only the hue is randomized each run.
const CLEAR_COLOR_SATURATION: f32 = 0.312;
const CLEAR_COLOR_LIGHTNESS: f32 = 0.365;

/// hud.tmx is 30 tiles wide and 4 tiles tall, each tile is 16x16 px.
const HUD_TILES_H: u32 = 4;
const HUD_TILE_SIZE: u32 = 16;
const HUD_LOGICAL_H: u32 = HUD_TILES_H * HUD_TILE_SIZE; // 64
const HUD_SCALE: f32 = 2.0;

/// Marker component for the HUD overlay camera.
#[derive(Component)]
pub struct HudCamera;

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(Color::hsl(
        0.0,
        CLEAR_COLOR_SATURATION,
        CLEAR_COLOR_LIGHTNESS,
    )));
    app.add_systems(Startup, (initialize_cameras, randomize_clear_color));
    app.add_systems(
        Update,
        (initialize_hud_rendering, update_camera, update_hud_viewport),
    );
    app.add_plugins(HierarchyPropagatePlugin::<RenderLayers>::new(Update));
}

fn randomize_clear_color(mut clear_color: ResMut<ClearColor>) {
    let hue = rand::rng().random_range(0.0f32..360.0f32);
    clear_color.0 = Color::hsl(hue, CLEAR_COLOR_SATURATION, CLEAR_COLOR_LIGHTNESS);
    info!("HSL = {:?}", clear_color.0);
}

/// Computes the HUD viewport for a given physical window width.
/// The HUD is scaled uniformly so its width matches the window width,
/// and placed at the top of the screen.
fn hud_viewport(physical_window_w: u32) -> Viewport {
    let vp_w = physical_window_w;
    let vp_h = (HUD_LOGICAL_H as f32 * HUD_SCALE).round() as u32;
    Viewport {
        physical_position: UVec2::ZERO, // top-left corner
        physical_size: UVec2::new(vp_w, vp_h),
        ..default()
    }
}

fn initialize_hud_rendering(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    hud_map_query: Query<Entity, With<HUD>>,
) {
    for map_created_message in map_created_reader.read() {
        if let Ok(hud_map_entity) = hud_map_query.get(map_created_message.origin) {
            commands
                .entity(hud_map_entity)
                .insert(Propagate(RenderLayers::from_layers(&[HUD_RENDER_LAYER])));
            return;
        }
    }
}

fn initialize_cameras(mut commands: Commands, window: Single<&Window>) {
    commands.spawn((
        Camera2d,
        DepthOfField::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: MIN_ZOOM_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Name::new("Hud Camera"),
        HudCamera,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0 / HUD_SCALE,
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            order: 1,
            viewport: Some(hud_viewport(window.physical_width())),
            ..default()
        },
        RenderLayers::layer(HUD_RENDER_LAYER),
    ));
}

/// Keeps the HUD viewport in sync when the window is resized.
fn update_hud_viewport(
    mut resize_events: MessageReader<WindowResized>,
    window: Single<&Window>,
    mut hud_camera: Single<&mut Camera, With<HudCamera>>,
) {
    if resize_events.read().last().is_none() {
        return;
    }
    hud_camera.viewport = Some(hud_viewport(window.physical_width()));
}

fn update_camera(
    mut set: ParamSet<(
        Single<(&mut Transform, &mut Projection), (With<Camera2d>, Without<RenderLayers>)>,
        Query<&Transform, With<Player>>,
    )>,
    time: Res<Time>,
) {
    // Calculate barycenter of all player positions
    let player_count = set.p1().count() as f32;
    if player_count == 0.0 {
        return;
    }

    let mut total_x = 0.0;
    let mut total_y = 0.0;

    // Collect player positions for barycenter and distance calculations
    let player_positions: Vec<Vec2> = {
        set.p1()
            .iter()
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
            .collect()
    };

    for position in &player_positions {
        total_x += position.x;
        total_y += position.y;
    }

    let barycenter = Vec2::new(total_x / player_count, total_y / player_count);

    // Calculate the maximum distance between any two players
    let max_distance = if player_positions.len() > 1 {
        let mut max_dist = 0.0;
        for i in 0..player_positions.len() {
            for j in (i + 1)..player_positions.len() {
                let distance = player_positions[i].distance(player_positions[j]);
                if distance > max_dist {
                    max_dist = distance;
                }
            }
        }
        max_dist
    } else {
        BASE_ZOOM_DISTANCE // Default distance for single player
    };

    // Calculate desired zoom scale based on player spread
    let zoom_factor = (max_distance / BASE_ZOOM_DISTANCE).clamp(0.5, 3.0);
    let target_scale = (MIN_ZOOM_SCALE * zoom_factor).clamp(MIN_ZOOM_SCALE, MAX_ZOOM_SCALE);

    // Update camera position to barycenter
    let (mut camera_transform, mut projection) = set.p0().into_inner();

    let direction = Vec3::new(barycenter.x, barycenter.y, camera_transform.translation.z);
    camera_transform
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());

    // Update zoom scale smoothly
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        ortho.scale = ortho
            .scale
            .lerp(target_scale, CAMERA_DECAY_RATE * time.delta_secs());
    }
}
