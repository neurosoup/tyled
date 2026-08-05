/*
 * Plugin for the beam-ability substrate.
 *
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<AbilityList>();
    app.register_type::<AbilityDescriptor>();

    app.insert_resource(PlayerLoadouts {
        player1: vec![AbilityDescriptor::SolarPanels],
        player2: vec![AbilityDescriptor::Overpenetration],
    });
}
