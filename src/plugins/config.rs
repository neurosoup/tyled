/*
 * Central game-tuning config. Every gameplay/timing/visual knob lives in
 * `assets/game_config.ron`; this module loads it into the `GameConfig` resource.
 */
use bevy::prelude::*;
use serde::Deserialize;

/// Embedded copy of the tuning file. The single source of truth: release builds
/// parse this at startup, dev builds fall back to it if the on-disk file is
/// missing/unparseable.
const EMBEDDED_CONFIG: &str = include_str!("../../assets/game_config.ron");

/// All tunable gameplay/timing/visual values, grouped by domain.
#[derive(Resource, Asset, Reflect, Clone, Deserialize)]
#[reflect(Resource)]
pub struct GameConfig {
    pub timing: TimingConfig,
    pub damage: DamageConfig,
    pub round: RoundConfig,
    pub camera: CameraConfig,
    pub player: PlayerConfig,
    pub animation: AnimationConfig,
    pub effects: EffectsConfig,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct TimingConfig {
    /// Milliseconds a direction must be held before the character starts auto-walking.
    pub move_repeat_delay_ms: u64,
    /// Milliseconds between steps while a direction is held (walk cadence and speed).
    pub move_repeat_rate_ms: u64,
    /// Milliseconds a diagonal must be held before it counts, filtering key-roll transients.
    pub diagonal_debounce_ms: u64,
    /// Seconds between beam step ticks. Lower = beams travel faster.
    pub beam_step_secs: f32,
    /// Milliseconds per step of the in-place turn animation.
    pub turn_step_ms: u64,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct DamageConfig {
    /// Damage per tick while standing on a hostile tile.
    pub standing: f32,
    /// Damage each time a character moves onto a hostile tile.
    pub on_enter: f32,
    /// Damage from a beam passing through a character.
    pub beam_contact: f32,
    /// Seconds between standing-damage ticks. Lower = damage stacks faster.
    pub tick_secs: f32,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct RoundConfig {
    /// Round length in whole seconds (the in-round countdown's starting value).
    pub round_duration_secs: u32,
    /// How many intro numbers count down before "GO!" (3 → "3", "2", "1", one per
    /// second, so the intro lasts `intro_count` seconds).
    pub intro_count: u8,
    /// Milliseconds the "GO!" banner scales up before it despawns.
    pub go_linger_ms: u64,
    /// Scale the "GO!" banner rushes to before despawning.
    pub go_end_scale: f32,
    /// Seconds the win banner lingers before the round loops.
    pub outcome_linger_secs: f32,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct CameraConfig {
    /// How fast the camera snaps to its target position. Higher = snappier.
    pub decay_rate: f32,
    /// How fast the zoom lerps between pixel-perfect levels. Higher = snappier.
    pub zoom_decay_rate: f32,
    /// Base distance for zoom math and the single-player default framing.
    pub base_zoom_distance: f32,
    /// Pixel-perfect zoom levels (1/n scales); the only valid zoom choices.
    pub zoom_levels: [f32; 4],
    /// Lower bound on the distance-to-zoom factor.
    pub zoom_min: f32,
    /// Upper bound on the distance-to-zoom factor.
    pub zoom_max: f32,
    /// Background clear-colour hue (0–360).
    pub bg_hue: f32,
    /// Background clear-colour saturation (0–1).
    pub bg_saturation: f32,
    /// Background clear-colour lightness (0–1).
    pub bg_lightness: f32,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct PlayerConfig {
    /// Starting and maximum player health.
    pub starting_health: f32,
    /// Starting beam charges = ground-tile count divided by this. Higher = fewer.
    pub beam_charges_divisor: u32,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct AnimationConfig {
    /// Milliseconds per frame of the player idle animations.
    pub player_idle_frame_ms: u32,
    /// Milliseconds per frame of the claimed-tile flip animations.
    pub tile_flip_frame_ms: u32,
    /// Milliseconds per frame of the HUD digit rolling-odometer animations.
    pub digit_roll_frame_ms: u32,
    /// Max cascade delay (seconds) for the tile-unclaim revert, scaled by distance.
    pub unclaim_cascade_secs: f32,
}

#[derive(Reflect, Clone, Deserialize)]
pub struct EffectsConfig {
    /// Milliseconds for the knockback slide tween.
    pub knockback_tween_ms: u64,
    /// Milliseconds for the damage colour-flash tween.
    pub damage_flash_ms: u64,
}

impl GameConfig {
    /// Parse the compile-time embedded config. Used verbatim in release builds and
    /// as the dev fallback.
    fn embedded() -> Self {
        ron::from_str(EMBEDDED_CONFIG).expect("embedded game_config.ron must parse")
    }
}

pub(crate) fn plugin(app: &mut App) {
    #[cfg(not(feature = "dev"))]
    {
        app.insert_resource(GameConfig::embedded());
    }

    #[cfg(feature = "dev")]
    {
        // Read from disk at runtime so edits don't force a recompile; fall back to
        // the embedded copy if the file is missing or malformed.
        let initial = std::fs::read_to_string("assets/game_config.ron")
            .ok()
            .and_then(|text| ron::from_str::<GameConfig>(&text).ok())
            .unwrap_or_else(GameConfig::embedded);
        app.insert_resource(initial);

        app.init_asset::<GameConfig>();
        app.register_asset_loader(dev::GameConfigLoader);
        app.add_systems(Startup, dev::load_config_asset);
        app.add_systems(Update, dev::apply_config_reload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The baked-in tuning file must stay a valid `GameConfig` so a malformed edit
    /// (bad syntax, missing/renamed field, wrong type) fails the build, not the
    /// player. Asserts only that it parses — never specific values, so tuning the
    /// RON never breaks this test.
    #[test]
    fn embedded_config_parses() {
        let _ = GameConfig::embedded();
    }
}

#[cfg(feature = "dev")]
mod dev {
    use super::GameConfig;
    use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext, io::Reader};
    use bevy::prelude::*;

    /// Handle to the hot-reloadable on-disk config asset.
    #[derive(Resource)]
    pub struct GameConfigHandle(#[allow(dead_code)] Handle<GameConfig>);

    #[derive(Default, TypePath)]
    pub struct GameConfigLoader;

    impl AssetLoader for GameConfigLoader {
        type Asset = GameConfig;
        type Settings = ();
        type Error = Box<dyn std::error::Error + Send + Sync>;

        async fn load(
            &self,
            reader: &mut dyn Reader,
            _settings: &(),
            _load_context: &mut LoadContext<'_>,
        ) -> Result<GameConfig, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let config = ron::de::from_bytes::<GameConfig>(&bytes)?;
            Ok(config)
        }

        fn extensions(&self) -> &[&str] {
            &["ron"]
        }
    }

    pub fn load_config_asset(mut commands: Commands, assets: Res<AssetServer>) {
        commands.insert_resource(GameConfigHandle(assets.load("game_config.ron")));
    }

    pub fn apply_config_reload(
        mut events: MessageReader<AssetEvent<GameConfig>>,
        configs: Res<Assets<GameConfig>>,
        mut config: ResMut<GameConfig>,
    ) {
        for event in events.read() {
            if let AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } = event
                && let Some(updated) = configs.get(*id)
            {
                *config = updated.clone();
            }
        }
    }
}
