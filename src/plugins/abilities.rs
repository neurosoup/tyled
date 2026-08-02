/*
 * Plugin for the beam-ability substrate.
 *
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<AbilityList>();
    app.register_type::<AbilityDescriptor>();

    // Slice 1 hardcoded loadouts (no draft UI yet): Solar Panels (A) vs
    // Overpenetration ± Lance (B). Edit these lists to swap P1/P2 kits between
    // runs.
    app.insert_resource(PlayerLoadouts {
        player1: vec![AbilityDescriptor::SolarPanels],
        player2: vec![AbilityDescriptor::Overpenetration, AbilityDescriptor::Lance],
        // player2: vec![AbilityDescriptor::Overpenetration], // Lance-ablation variant
    });

    // Slice 1: on_resolve / on_claim descriptor resolvers land here.
}
