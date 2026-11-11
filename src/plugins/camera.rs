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
/// Height of the HUD strip in physical pixels.
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

fn initialize_cameras(mut commands: Commands, window: Single<&Window>, config: Res<GameConfig>) {
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

/// Smallest level that is at least `preferred` and `required`, or the largest level.
fn choose_zoom_level(preferred: f32, required: f32, levels: &[f32]) -> f32 {
    let wanted = preferred.max(required);
    let largest = levels.iter().copied().fold(f32::MIN, f32::max);
    levels
        .iter()
        .copied()
        .filter(|&level| level >= wanted)
        .fold(largest, f32::min)
}

fn update_camera(
    mut set: ParamSet<(
        Single<(&mut Transform, &mut Projection), (With<Camera2d>, Without<RenderLayers>)>,
        Query<&Transform, With<Character>>,
    )>,
    time: Res<Time>,
    config: Res<GameConfig>,
    window: Single<&Window>,
    map_info: Option<Res<MapInfo>>,
) {
    let zoom_levels = config.camera.zoom_levels;
    let character_positions: Vec<Vec2> = set
        .p1()
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();
    if character_positions.is_empty() {
        return;
    }

    let mut box_min = character_positions[0];
    let mut box_max = character_positions[0];
    for position in &character_positions {
        box_min = box_min.min(*position);
        box_max = box_max.max(*position);
    }
    let box_center = (box_min + box_max) / 2.0;

    // Largest distance between any two players.
    let max_distance = if character_positions.len() > 1 {
        let mut max_dist = 0.0;
        for i in 0..character_positions.len() {
            for j in (i + 1)..character_positions.len() {
                max_dist = character_positions[i]
                    .distance(character_positions[j])
                    .max(max_dist);
            }
        }
        max_dist
    } else {
        config.camera.base_zoom_distance
    };

    let zoom_factor = (max_distance / config.camera.base_zoom_distance)
        .clamp(config.camera.zoom_min, config.camera.zoom_max);
    let continuous_scale = zoom_levels[0] * zoom_factor;

    // The HUD height in logical pixels, the unit of the orthographic scale.
    let hud_logical_px = HUD_VIEWPORT_H / window.scale_factor();

    // One tile of margin keeps the player sprite fully visible.
    let margin = map_info
        .map(|info| Vec2::new(info.grid_size.x, info.grid_size.y))
        .unwrap_or(Vec2::splat(16.0));
    let box_size = (box_max - box_min) + margin * 2.0;
    let visible_w = window.width().max(1.0);
    let visible_h = (window.height() - hud_logical_px).max(1.0);
    let required_scale = (box_size.x / visible_w).max(box_size.y / visible_h);

    let target_scale = choose_zoom_level(continuous_scale, required_scale, &zoom_levels);

    let (mut camera_transform, mut projection) = set.p0().into_inner();

    // Shift the camera up so the players are centred in the area below the HUD.
    let current_scale = match projection.as_ref() {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => zoom_levels[0],
    };
    let hud_offset_y = hud_logical_px / 2.0 * current_scale;

    let direction = Vec3::new(
        box_center.x,
        box_center.y + hud_offset_y,
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

#[cfg(test)]
mod tests {
    use super::choose_zoom_level;

    const LEVELS: [f32; 4] = [0.25, 1.0 / 3.0, 0.5, 1.0];

    #[test]
    fn preferred_wins() {
        assert_eq!(choose_zoom_level(0.4, 0.1, &LEVELS), 0.5);
    }

    #[test]
    fn required_forces_wider_level() {
        assert_eq!(choose_zoom_level(0.25, 0.4, &LEVELS), 0.5);
    }

    #[test]
    fn nothing_fits_gives_largest() {
        assert_eq!(choose_zoom_level(0.25, 3.0, &LEVELS), 1.0);
    }

    #[test]
    fn exact_match_at_boundary() {
        assert_eq!(choose_zoom_level(0.5, 0.2, &LEVELS), 0.5);
    }

    #[test]
    fn unsorted_levels() {
        assert_eq!(
            choose_zoom_level(0.3, 0.1, &[1.0, 0.25, 0.5, 1.0 / 3.0]),
            1.0 / 3.0
        );
    }
}
