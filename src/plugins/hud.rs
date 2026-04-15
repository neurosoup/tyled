use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, update_hp);
}

fn update_hp(
    players: Query<(&Health, &Player), With<DamageEffectTarget>>,
    mut hp_bars: Query<(&HPBar, &mut Transform)>,
) {
    for (health, player) in &players {
        // Apply damage effect to HP bars
        for (hp_bar, mut transform) in &mut hp_bars {
            if hp_bar.player_id == player.player_id {
                let ratio = health.ratio();
                transform.scale.x = transform.scale.x.lerp(ratio, 0.05);
                if transform.scale.x <= 0.001 {
                    transform.scale.x = 0.0;
                }
            }
        }
    }
}
