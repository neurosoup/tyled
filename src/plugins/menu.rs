/*
 * The pre-round main menu: lets players pick a matchup (which seats are bot-
 * controlled) before any map loads. The confirmed choice is written into the
 * `GameConfig` resource's `ControllersConfig`, then `AppState` transitions to
 * `InRound`, unblocking `maps::load_maps`. Drawn with the `text` plugin's
 * `spawn_label` onto the overlay camera (owned by the `camera` plugin).
 */
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::prelude::*;

/// The app's top-level lifecycle, gating map loading behind the main menu.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Picking a matchup; no map has loaded yet.
    #[default]
    MainMenu,
    /// The matchup is confirmed and round gameplay proceeds as normal.
    InRound,
}

/// A selectable matchup, mapping to which seats are bot-controlled.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MatchupChoice {
    PlayerVsPlayer,
    PlayerVsBot,
    BotVsPlayer,
}

impl MatchupChoice {
    /// `(player1_bot, player2_bot)` for this matchup.
    pub fn controllers(&self) -> (bool, bool) {
        match self {
            MatchupChoice::PlayerVsPlayer => (false, false),
            MatchupChoice::PlayerVsBot => (false, true),
            MatchupChoice::BotVsPlayer => (true, false),
        }
    }
}

const MENU_ENTRIES: [(MatchupChoice, &str); 3] = [
    (MatchupChoice::PlayerVsPlayer, "1P VS 2P"),
    (MatchupChoice::PlayerVsBot, "1P VS BOT"),
    (MatchupChoice::BotVsPlayer, "BOT VS 2P"),
];

/// Index of the currently-highlighted menu row.
#[derive(Resource, Default)]
pub struct MenuSelection(pub usize);

/// Tags every spawned menu row entity, for bulk despawn.
#[derive(Component)]
pub struct MenuLabel;

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<AppState>();
    app.init_resource::<MenuSelection>();
    app.add_systems(
        Update,
        (ensure_menu_spawned, update_selection, confirm_selection)
            .chain()
            .run_if(in_state(AppState::MainMenu)),
    );
}

/// Spawns one label per menu row, prefixing the selected row with the font
/// atlas's selector glyph (`>`) and every other row with a leading space so
/// alignment stays identical either way.
fn spawn_rows(commands: &mut Commands, font: &FontAtlas, selection: &MenuSelection) {
    for (i, (_, text)) in MENU_ENTRIES.iter().enumerate() {
        let prefix = if i == selection.0 { ">" } else { " " };
        let label = spawn_label(
            commands,
            font,
            &format!("{prefix}{text}"),
            Transform::from_xyz(0.0, 24.0 - i as f32 * 24.0, 0.0),
            RenderLayers::layer(OVERLAY_RENDER_LAYER),
        );
        commands.entity(label).insert(MenuLabel);
    }
}

/// Draws the initial rows the first time `FontAtlas` is available. Gated on
/// `Update` rather than `OnEnter`: the default state's `OnEnter` fires before
/// `Startup` systems (like `text::setup_font_atlas`) have run, so `FontAtlas`
/// doesn't exist there yet.
fn ensure_menu_spawned(
    mut commands: Commands,
    font: Option<Res<FontAtlas>>,
    selection: Res<MenuSelection>,
    labels: Query<(), With<MenuLabel>>,
) {
    let Some(font) = font else {
        return;
    };
    if !labels.is_empty() {
        return;
    }
    spawn_rows(&mut commands, &font, &selection);
}

/// Moves the cursor with W/S or the arrow keys (shared cursor, either works),
/// clamped to the entry range, and respawns the rows on any change.
fn update_selection(
    mut commands: Commands,
    font: Res<FontAtlas>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MenuSelection>,
    labels: Query<Entity, With<MenuLabel>>,
) {
    let up = keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp);
    let down = keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown);

    let mut changed = false;
    if up && selection.0 > 0 {
        selection.0 -= 1;
        changed = true;
    }
    if down && selection.0 < MENU_ENTRIES.len() - 1 {
        selection.0 += 1;
        changed = true;
    }

    if !changed {
        return;
    }

    for entity in &labels {
        commands.entity(entity).despawn();
    }
    spawn_rows(&mut commands, &font, &selection);
}

/// Confirms the highlighted matchup with Q or `/`, applies it to the
/// controllers config, and hands off to `InRound`.
fn confirm_selection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    mut config: ResMut<GameConfig>,
    labels: Query<Entity, With<MenuLabel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !(keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Slash)) {
        return;
    }

    let (player1_bot, player2_bot) = MENU_ENTRIES[selection.0].0.controllers();
    config.controllers.player1_bot = player1_bot;
    config.controllers.player2_bot = player2_bot;

    for entity in &labels {
        commands.entity(entity).despawn();
    }

    next_state.set(AppState::InRound);
}
