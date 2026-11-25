/*
 * Plugin for the beam-ability substrate.
 *
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<AbilityList>();
    app.register_type::<AbilityDescriptor>();

    let launch = app.world().resource::<GameConfig>().launch.clone();
    app.insert_resource(PlayerLoadouts {
        player1: launch
            .loadout_p1
            .unwrap_or_else(|| vec![AbilityDescriptor::SolarPanels]),
        player2: launch
            .loadout_p2
            .unwrap_or_else(|| vec![AbilityDescriptor::BorderGrinder]),
    });
}
