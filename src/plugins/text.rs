/*
 * Bitmap-font text-rendering service. Loads the shared font atlas
 * (`assets/font.png`) into the `FontAtlas` resource at startup and provides
 * `spawn_label`, which composes a string into a row of per-glyph sprites. Any
 * feature can use it — today the `overlay` plugin's intro countdown; later the
 * win banner and shop price labels. Not tied to any one camera: the caller
 * passes the `RenderLayers` the glyphs should render on.
 */
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::prelude::*;

/// Number of columns in the font atlas grid.
pub const FONT_COLS: u32 = 16;
/// Number of rows in the font atlas grid.
pub const FONT_ROWS: u32 = 3;
/// Per-cell pixel size in the atlas.
pub const FONT_CELL: UVec2 = UVec2::splat(16);
/// Horizontal advance between adjacent glyph origins, in atlas pixel units.
pub const FONT_ADVANCE: f32 = 16.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_font_atlas);
}

/// Handles + layout for the shared bitmap font, inserted once at startup.
#[derive(Resource, Clone)]
pub struct FontAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Loads `assets/font.png` and inserts the `FontAtlas` resource.
fn setup_font_atlas(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = assets.load("font.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        FONT_CELL, FONT_COLS, FONT_ROWS, None, None,
    ));
    commands.insert_resource(FontAtlas { image, layout });
}

/// Maps a character to its cell index in `assets/font.png`. The atlas is a
/// 16×3 grid laid out as: row 0 = `A`–`P` (0–15), row 1 = `Q`–`Z` (16–25) then
/// 6 blank cells, row 2 = `0`–`9` (32–41), `!` `?` `.` `:` (42–45), 2 blanks.
/// Returns `None` for spaces and any unmapped character (including `¢`, absent
/// from the current atlas) — those advance the cursor but draw no sprite.
/// Lookup is case-insensitive; the atlas is uppercase-only.
pub fn glyph_index(c: char) -> Option<usize> {
    match c.to_ascii_uppercase() {
        c @ 'A'..='Z' => Some(c as usize - 'A' as usize),
        c @ '0'..='9' => Some(32 + (c as usize - '0' as usize)),
        '!' => Some(42),
        '?' => Some(43),
        '.' => Some(44),
        ':' => Some(45),
        _ => None,
    }
}

/// Spawns a horizontally-centered run of glyph sprites for `text`, returning the
/// parent entity. `transform` positions the label's center point; `render_layers`
/// selects which camera renders the glyphs.
///
/// The `render_layers` is attached to every glyph sprite **directly**: render
/// layers do not propagate parent→child in this codebase without `Propagate`, so
/// a layer set only on the parent would leave the glyphs on the default layer.
/// Despawn the returned entity recursively to remove the glyph children with it.
pub fn spawn_label(
    commands: &mut Commands,
    font: &FontAtlas,
    text: &str,
    transform: Transform,
    render_layers: RenderLayers,
) -> Entity {
    let parent = commands
        .spawn((
            Name::new(format!("Label: {text}")),
            transform,
            Visibility::default(),
        ))
        .id();

    let count = text.chars().count() as f32;
    for (i, c) in text.chars().enumerate() {
        let Some(index) = glyph_index(c) else {
            continue;
        };
        let x = (i as f32 - (count - 1.0) / 2.0) * FONT_ADVANCE;
        let glyph = commands
            .spawn((
                Sprite::from_atlas_image(
                    font.image.clone(),
                    TextureAtlas {
                        layout: font.layout.clone(),
                        index,
                    },
                ),
                Transform::from_xyz(x, 0.0, 0.0),
                render_layers.clone(),
            ))
            .id();
        commands.entity(parent).add_child(glyph);
    }

    parent
}
