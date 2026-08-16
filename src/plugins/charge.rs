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
    mut players: Query<(Entity, &AbilityList, &ClaimedTileCount, &mut BeamCharges)>,
    mut charge_regen_writer: MessageWriter<ChargeRegen>,
) {
    solar_panels_timer.0.tick(time.delta());
    if !solar_panels_timer.0.is_finished() {
        return;
    }
    for (entity, abilities, tile_count, mut charges) in &mut players {
        if !abilities.0.contains(&AbilityDescriptor::SolarPanels) {
            continue;
        }
        let gained = tile_count.current / config.charge.solar_panels_tiles_per_charge;
        if gained == 0 {
            continue;
        }
        charges.current = (charges.current + gained).min(charges.max);
        charge_regen_writer.write(ChargeRegen {
            owner: entity,
            amount: gained,
        });
    }
}
