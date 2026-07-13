/*
 * Plugin for claiming tiles when a beam resolves.
 *
 * Owns the authoritative `ClaimedTile::owner` write and the `TileClaimed`
 * signal. It reads `BeamResolved` (emitted by the beam plugin) rather than
 * querying beams directly — the only coupling to the beam plugin is that
 * message.
 */
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, claim_tile);
    // Stage F2: on_resolve / on_claim descriptor resolvers land here.
}

fn claim_tile(
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    mut claimed_tiles: Query<&mut ClaimedTile>,
    map_info: Res<MapInfo>,
    mut tile_claimed_writer: MessageWriter<TileClaimed>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        if let Some(claimed_entity) = map_info
            .claimed_entities
            .get(&tile_claimed_message.position)
        {
            if let Ok(mut claimed_tile) = claimed_tiles.get_mut(*claimed_entity) {
                let old_owner = claimed_tile.owner;
                claimed_tile.owner = Some(tile_claimed_message.owner);
                tile_claimed_writer.write(TileClaimed {
                    position: tile_claimed_message.position,
                    old_owner,
                    new_owner: tile_claimed_message.owner,
                });
            }
        }
    }
}
