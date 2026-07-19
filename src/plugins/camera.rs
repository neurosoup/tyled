/*
 * Camera setup and runtime updates (world, HUD, and overlay cameras).
 */
use crate::prelude::*;
use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    camera::{ScalingMode, Viewport, visibility::RenderLayers},
    prelude::*,
    render::view::Msaa,
    window::WindowResized,
};
use bevy_ecs_tiled::prelude::{MapCreated, TiledEvent, TiledImage, TiledObject};
use bevy_smooth_pixel_camera::components::ViewportCamera;
use bevy_smooth_pixel_camera::prelude::*;
use rand::Rng;

const HUD_TILES_H: u32 = 4;
const HUD_TILE_SIZE: u32 = 16;
const HUD_LOGICAL_H: u32 = HUD_TILES_H * HUD_TILE_SIZE; // 64
const HUD_SCALE: f32 = 2.0;
/// Physical-pixel height of the HUD strip at the top of the window
/// (`hud_viewport`'s `vp_h`). The game world is shifted down by half this
/// so barycentre-centred players near the top of the arena aren't occluded.
const HUD_VIEWPORT_H: f32 = HUD_LOGICAL_H as f32 * HUD_SCALE; // 128

pub const HUD_RENDER_LAYER: usize = 1;
pub const LEVEL_RENDER_LAYER: usize = 2;
pub const OVERLAY_RENDER_LAYER: usize = 3;
const OVERLAY_VIEWPORT_HEIGHT: f32 = 180.0;

pub(crate) fn plugin(app: &mut App) {
    let (hue, saturation, lightness) = {
        let camera = &app.world().resource::<GameConfig>().camera;
        (camera.bg_hue, camera.bg_saturation, camera.bg_lightness)
    };
    app.insert_resource(ClearColor(Color::hsl(hue, saturation, lightness)));
    app.add_systems(Startup, initialize_cameras);
    app.add_systems(
        Update,
        (
            sync_clear_color,
            initialize_hud_rendering,
            update_camera,
            update_hud_viewport,
            disable_viewport_camera_msaa,
        ),
    );
    app.add_plugins((
        HierarchyPropagatePlugin::<RenderLayers>::new(Update),
        PixelCameraPlugin,
    ));
}

/// Keeps the background clear colour in sync with the config (so it hot-reloads in dev).
fn sync_clear_color(config: Res<GameConfig>, mut clear_color: ResMut<ClearColor>) {
    if config.is_changed() {
        clear_color.0 = Color::hsl(
            config.camera.bg_hue,
            config.camera.bg_saturation,
            config.camera.bg_lightness,
        );
    }
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

fn initialize_cameras(
    mut commands: Commands,
    window: Single<&Window>,
    config: Res<GameConfig>,
) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scale: config.camera.zoom_levels[0],
            ..OrthographicProjection::default_2d()
        }),
        IsDefaultUiCamera,
        PixelCamera {
            viewport_size: ViewportScalingMode::PixelSize(1.0),
            smoothing: false,
            viewport_layers: RenderLayers::layer(LEVEL_RENDER_LAYER),
            viewport_order: 2,
        },
    ));

    commands.spawn((
        Name::new("Hud Camera"),
        HudMap,
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            // Pixel-perfect 2× at spawn; otherwise the HUD renders at scale 1.0 until the
            // first WindowResized fires (see update_hud_viewport).
            scale: window.scale_factor() / HUD_SCALE,
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            order: 3,
            viewport: Some(hud_viewport(window.physical_width())),
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(HUD_RENDER_LAYER),
    ));

    commands.spawn((
        Name::new("Overlay Camera"),
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: OVERLAY_VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            order: 4,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(OVERLAY_RENDER_LAYER),
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
        // Pixel-perfect: physical px per world unit = scale_factor / scale, so scale =
        // scale_factor / HUD_SCALE renders every 16px tile at exactly 16 * HUD_SCALE physical px.
        ortho.scale = window.scale_factor() / HUD_SCALE;
    }
}

/// The ViewportCamera child spawned by PixelCamera's on_add hook has no Msaa::Off requirement,
/// so it defaults to Sample4. This conflicts with the offscreen texture (Msaa::Off) and causes
/// a wgpu validation error. Patch it as soon as the entity appears.
fn disable_viewport_camera_msaa(
    mut commands: Commands,
    viewport_cameras: Query<Entity, Added<ViewportCamera>>,
) {
    for entity in &viewport_cameras {
        commands.entity(entity).insert(Msaa::Off);
    }
}

fn update_camera(
    mut set: ParamSet<(
        Single<(&mut Transform, &mut Projection), (With<Camera2d>, Without<RenderLayers>)>,
        Query<&Transform, With<Character>>,
    )>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let zoom_levels = config.camera.zoom_levels;
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
        config.camera.base_zoom_distance // Default distance for single player
    };

    // Snap to the nearest pixel-perfect zoom level.
    let zoom_factor = (max_distance / config.camera.base_zoom_distance)
        .clamp(config.camera.zoom_min, config.camera.zoom_max);
    let continuous_scale = zoom_levels[0] * zoom_factor;
    let target_scale = zoom_levels
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

    // Shift the framing down so the barycentre sits in the centre of the region
    // *below* the HUD strip rather than the centre of the whole window, keeping
    // top-of-arena players clear of the HUD overlay. The HUD covers a fixed
    // HUD_VIEWPORT_H physical pixels at the top; world-per-physical-pixel equals
    // the orthographic scale, so this offset tracks the current zoom level.
    // Only `scale` affects pixel-perfection — translation is snapped/subpixel-
    // blitted by the pixel camera — so this shift is safe at any zoom.
    let current_scale = match projection.as_ref() {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => zoom_levels[0],
    };
    let hud_offset_y = (HUD_VIEWPORT_H / 2.0) * current_scale;

    let direction = Vec3::new(
        barycenter.x,
        barycenter.y + hud_offset_y,
        camera_transform.translation.z,
    );
    camera_transform.translation.smooth_nudge(
        &direction,
        config.camera.decay_rate,
        time.delta_secs(),
    );

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        if (ortho.scale - target_scale).abs() < 0.001 {
            ortho.scale = target_scale;
        } else {
            ortho.scale.smooth_nudge(
                &target_scale,
                config.camera.zoom_decay_rate,
                time.delta_secs(),
            );
        }
    }
}
