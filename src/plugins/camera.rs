use crate::prelude::*;
use bevy::{
    post_process::{bloom::Bloom, dof::DepthOfField, motion_blur::MotionBlur},
    prelude::*,
};

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 0.5;
/// Minimum zoom scale (zoomed out)
const MIN_ZOOM_SCALE: f32 = 0.33;
/// Maximum zoom scale (zoomed in)
const MAX_ZOOM_SCALE: f32 = 2.0;
/// Base distance for zoom calculations
const BASE_ZOOM_DISTANCE: f32 = 200.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, initialize_camera);
    app.add_systems(Update, update_camera);
}

fn initialize_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        DepthOfField::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: MIN_ZOOM_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn update_camera(
    camera_query: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    players: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let (mut camera_transform, mut projection) = camera_query.into_inner();

    // Calculate barycenter of all player positions
    if players.is_empty() {
        return;
    }

    let mut total_x = 0.0;
    let mut total_y = 0.0;
    let player_count = players.iter().count() as f32;

    // Collect player positions for barycenter and distance calculations
    let player_positions: Vec<Vec2> = players
        .iter()
        .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
        .collect();

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
