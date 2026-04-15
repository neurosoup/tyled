use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, update_hp);
}

pub fn update_hp(players: Query<(&Player, &Health), Changed<Health>>) {
    for (player, health) in &players {
        match player.player_id {
            0 => info!("Player 0 health: {}", health.current),
            1 => info!("Player 1 health: {}", health.current),
            _ => {}
        }
    }
}
