use agb::display::object::{Graphics, Tag};
use agb::display::palette16::Palette16;
use agb::display::tile_data::TileData;
use agb::{include_aseprite, include_background_gfx};

include_background_gfx!(bg, "0a0b0c", DATA => "assets/sprites/background.png");
pub static TILEDATA: &TileData = &bg::DATA;
pub static PALETTES: &[Palette16] = bg::PALETTES;
pub static SPRITES: &Graphics = include_aseprite!("assets/sprites/sprites.aseprite");

#[allow(dead_code)]
pub mod tags {

    use super::*;

    pub static MAP_BLOCK_SPRITE: &Tag = SPRITES.tags().get("Block");
    pub static MAP_UP_MIRROR: &Tag = SPRITES.tags().get("UpMirror");
    // Down mirror = x-flipped
    pub static MAP_HORIZ_MIRROR: &Tag = SPRITES.tags().get("HorizMirror");
    pub static MAP_VERT_MIRROR: &Tag = SPRITES.tags().get("VertMirror");
    pub static MAP_HORIZ_PIPE: &Tag = SPRITES.tags().get("HorizPipe");
    pub static MAP_VERT_PIPE: &Tag = SPRITES.tags().get("VertPipe");
    pub static PLAYERS: &[&Tag] = &[
        SPRITES.tags().get("P1"),
        SPRITES.tags().get("P2"),
        SPRITES.tags().get("P3"),
        SPRITES.tags().get("P4"),
    ];
}
