/*
 * Plugin for camera behavior and viewport management (HUD).
 */
use crate::prelude::*;
use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    camera::{Viewport, visibility::RenderLayers},
    prelude::*,
    window::WindowResized,
};
use bevy_ecs_tiled::prelude::{MapCreated, TiledEvent, TiledImage, TiledObject};
use bevy_smooth_pixel_camera::prelude::*;
use rand::Rng;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 4.0;
/// How quickly the zoom lerps between pixel-perfect levels.
const ZOOM_DECAY_RATE: f32 = 8.0;
/// Base distance for zoom calculations
const BASE_ZOOM_DISTANCE: f32 = 150.0;

/// Pixel-perfect zoom levels (1/n scales). With PixelSize(1.0), integer world positions
/// map to integer screen pixels at any 1/n scale, so these are the only valid choices.
const ZOOM_LEVELS: [f32; 4] = [1.0 / 4.0, 1.0 / 3.0, 1.0 / 2.0, 1.0];

// HSL equivalent of srgb_u8(100, 122, 64): saturation and lightness are fixed,
// only the hue is randomized each run.
const CLEAR_COLOR_SATURATION: f32 = 0.312;
const CLEAR_COLOR_LIGHTNESS: f32 = 0.365;

const HUD_MAP_WIDTH: u32 = 48;
const HUD_TILES_H: u32 = 4;
const HUD_TILE_SIZE: u32 = 16;
const HUD_LOGICAL_H: u32 = HUD_TILES_H * HUD_TILE_SIZE; // 64
const HUD_SCALE: f32 = 2.0;

pub const HUD_RENDER_LAYER: usize = 1;

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
    app.add_plugins((
        HierarchyPropagatePlugin::<RenderLayers>::new(Update),
        PixelCameraPlugin,
    ));
}

fn randomize_clear_color(mut clear_color: ResMut<ClearColor>) {
    let hue = 235.0; //rand::rng().random_range(0.0f32..360.0f32);
    // clear_color.0 = Color::hsl(hue, CLEAR_COLOR_SATURATION, CLEAR_COLOR_LIGHTNESS);
    clear_color.0 = Color::hsl(hue, 0.28, 0.18);
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
    hud_map_query: Query<Entity, With<HudMap>>,
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
        Projection::Orthographic(OrthographicProjection {
            scale: ZOOM_LEVELS[0],
            ..OrthographicProjection::default_2d()
        }),
        PixelCamera {
            viewport_size: ViewportScalingMode::PixelSize(1.0),
            smoothing: false,
            // Use layer 2 for the pixel viewport to avoid colliding with HUD_RENDER_LAYER (1).
            viewport_layers: RenderLayers::layer(2),
            viewport_order: 2,
        },
    ));

    commands.spawn((
        Name::new("Hud Camera"),
        HudMap,
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection {
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            // Must be > viewport_order (2) so HUD composites on top of the game world.
            order: 3,
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
    (mut hud_camera, mut projection): (
        Single<&mut Camera, With<HudMap>>,
        Single<&mut Projection, With<HudMap>>,
    ),
) {
    if resize_events.read().last().is_none() {
        return;
    }
    hud_camera.viewport = Some(hud_viewport(window.physical_width()));
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        let hud_physical_w = window.physical_width() as f32;
        let hud_content_w = (HUD_MAP_WIDTH * HUD_TILE_SIZE) as f32;
        ortho.scale = hud_content_w / hud_physical_w;
    }
}

fn update_camera(
    mut set: ParamSet<(
        Single<(&mut Transform, &mut Projection), (With<Camera2d>, Without<RenderLayers>)>,
        Query<&Transform, With<Character>>,
    )>,
    time: Res<Time>,
) {
    // Calculate barycenter of all player positions
    let character_count = set.p1().count() as f32;
    if character_count == 0.0 {
        return;
    }

    let mut total_x = 0.0;
    let mut total_y = 0.0;

    // Collect characters positions for barycenter and distance calculations
    let character_positions: Vec<Vec2> = {
        set.p1()
            .iter()
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
            .collect()
    };

    for position in &character_positions {
        total_x += position.x;
        total_y += position.y;
    }

    let barycenter = Vec2::new(total_x / character_count, total_y / character_count);

    // Calculate the maximum distance between any two players
    let max_distance = if character_positions.len() > 1 {
        let mut max_dist = 0.0;
        for i in 0..character_positions.len() {
            for j in (i + 1)..character_positions.len() {
                let distance = character_positions[i].distance(character_positions[j]);
                if distance > max_dist {
                    max_dist = distance;
                }
            }
        }
        max_dist
    } else {
        BASE_ZOOM_DISTANCE // Default distance for single player
    };

    // Snap to the nearest pixel-perfect zoom level.
    let zoom_factor = (max_distance / BASE_ZOOM_DISTANCE).clamp(0.5, 3.0);
    let continuous_scale = ZOOM_LEVELS[0] * zoom_factor;
    let target_scale = ZOOM_LEVELS
        .iter()
        .copied()
        .min_by(|&a, &b| {
            (a - continuous_scale)
                .abs()
                .total_cmp(&(b - continuous_scale).abs())
        })
        .unwrap();

    // Update camera position to barycenter
    let (mut camera_transform, mut projection) = set.p0().into_inner();

    let direction = Vec3::new(barycenter.x, barycenter.y, camera_transform.translation.z);
    camera_transform
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        if (ortho.scale - target_scale).abs() < 0.001 {
            ortho.scale = target_scale;
        } else {
            ortho
                .scale
                .smooth_nudge(&target_scale, ZOOM_DECAY_RATE, time.delta_secs());
        }
    }
}
