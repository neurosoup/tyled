/*
 * Plugin for the beam-ability substrate.
 *
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<AbilityList>();
    app.register_type::<AbilityDescriptor>();

    // Stage F2 hardcoded loadouts (no draft UI yet). Edit these lists to swap
    // P1/P2 kits between runs; the default hands Backfill to both, reproducing
    // today's contextual inverted mode. Empty lists give the Straight-only
    // layer-1 control.
    app.insert_resource(PlayerLoadouts {
        player1: vec![/*AbilityDescriptor::Backfill*/],
        player2: vec![/*AbilityDescriptor::Backfill*/],
    });

    // Slice 1: on_resolve / on_claim descriptor resolvers land here.
}
