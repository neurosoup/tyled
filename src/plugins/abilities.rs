/*
 * Plugin for the beam-ability substrate.
 *
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<AbilityList>();
    app.register_type::<AbilityDescriptor>();
    // Stage F2: on_resolve / on_claim descriptor resolvers land here.
}
