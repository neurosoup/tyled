/*
 * Plugin for BeamCharges economy: regen, refund, and cost policy resolvers.
 */
use crate::prelude::*;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_solar_panels_timer);
    app.add_systems(
        Update,
        regen_charges_from_solar_panels.run_if(in_state(RoundPhase::Playing)),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, resync_solar_panels_timer);
}

#[derive(Resource)]
pub struct SolarPanelsTimer(Timer);

fn setup_solar_panels_timer(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(SolarPanelsTimer(Timer::from_seconds(
        config.charge.solar_panels_tick_secs,
        TimerMode::Repeating,
    )));
}

#[cfg(feature = "dev")]
fn resync_solar_panels_timer(config: Res<GameConfig>, timer: Option<ResMut<SolarPanelsTimer>>) {
    if config.is_changed()
        && let Some(mut timer) = timer
    {
        timer.0.set_duration(std::time::Duration::from_secs_f32(
            config.charge.solar_panels_tick_secs,
        ));
    }
}

fn regen_charges_from_solar_panels(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut solar_panels_timer: ResMut<SolarPanelsTimer>,
    map_info: Res<MapInfo>,
    mut players: Query<(Entity, &AbilityList, &ClaimedTileCount, &mut BeamCharges)>,
    mut charge_regen_writer: MessageWriter<ChargeRegen>,
) {
    solar_panels_timer.0.tick(time.delta());
    if !solar_panels_timer.0.is_finished() {
        return;
    }

    // A charge is only useful up to a claim on a tile nobody owns yet, so cap
    // regen at the board's remaining unclaimed count — otherwise a wide-board
    // Solar Panels stack keeps banking charges it can never spend on a fresh
    // claim once the board's mostly settled.
    let total_tiles = map_info.ground_entities.len() as u32;
    let claimed_tiles: u32 = players.iter().map(|(.., count, _)| count.current).sum();
    let unclaimed_tiles = total_tiles.saturating_sub(claimed_tiles);

    for (entity, abilities, tile_count, mut charges) in &mut players {
        if !abilities.0.contains(&AbilityDescriptor::SolarPanels) {
            continue;
        }
        let gained = tile_count.current / config.charge.solar_panels_tiles_per_charge;
        if gained == 0 {
            continue;
        }
        let new_current = (charges.current + gained)
            .min(charges.max)
            .min(unclaimed_tiles);
        if new_current == charges.current {
            continue;
        }
        let amount = new_current - charges.current;
        charges.current = new_current;
        charge_regen_writer.write(ChargeRegen { owner: entity, amount });
    }
}
