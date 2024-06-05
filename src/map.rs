use core::hash::Hash;

use agb::display::{
    object::{OamManaged, Object},
    tiled::{MapLoan, RegularMap, TileSetting, VRamManager},
    HEIGHT, WIDTH,
};
use alloc::{string::String, vec::Vec};

mod generation;
use crate::{graphics::*, RectExt, RectType};
pub use generation::*;
mod tiles;
pub use tiles::*;

use crate::{VectType, N};

const BUFFER_TILES: i32 = 1;
const TILE_SIZE: i32 = 8;
const MAP_BYTE_WIDTH: usize = {
    let screen_tile_width = WIDTH / TILE_SIZE;
    let map_width = screen_tile_width - 2 * BUFFER_TILES;
    (map_width / 2) as usize
};
const MAP_WIDTH: usize = MAP_BYTE_WIDTH * 2;
const MAP_HEIGHT: usize = ((HEIGHT / TILE_SIZE) - 2 * BUFFER_TILES) as usize;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BaseMap {
    data: [[u8; MAP_BYTE_WIDTH]; MAP_HEIGHT],
    spawns: [(usize, usize); 4],
}

impl BaseMap {
    pub const fn from_raw(
        data: [[MapTile; MAP_WIDTH]; MAP_HEIGHT],
        spawns: [(usize, usize); 4],
    ) -> Self {
        let mut buffer = [[0u8; MAP_BYTE_WIDTH]; MAP_HEIGHT];
        let mut xidx = 0;
        while xidx < MAP_WIDTH {
            let mut yidx = 0;
            while yidx < MAP_HEIGHT {
                let cur_tile = data[yidx][xidx];
                let byte_xidx = xidx / 2;
                let base_mask = cur_tile.to_u8();

                let mut cur_byte = buffer[yidx][byte_xidx];
                if xidx % 2 == 0 {
                    cur_byte |= base_mask << 4;
                } else {
                    cur_byte |= base_mask;
                }
                buffer[yidx][byte_xidx] = cur_byte;
                yidx += 1;
            }
            xidx += 1;
        }
        Self {
            data: buffer,
            spawns,
        }
    }
    #[allow(dead_code)]
    pub fn flip(&mut self, x: usize, y: usize) {
        //TODO: Optimize
        self.set(x, y, self.get(x, y).flipped());
    }
    pub fn set(&mut self, x: usize, y: usize, tile: MapTile) {
        let elm = self.data[y][x / 2];
        let mask = tile as u8;
        let nelm = if x % 2 == 0 {
            (mask << 4) | (elm & 0x0F)
        } else {
            (elm & 0xF0) | mask
        };
        self.data[y][x / 2] = nelm;
    }
    pub const fn with(mut self, x: usize, y: usize, tile: MapTile) -> Self {
        let elm = self.data[y][x / 2];
        let mask = tile as u8;
        let nelm = if x % 2 == 0 {
            (mask << 4) | (elm & 0x0F)
        } else {
            (elm & 0xF0) | mask
        };
        self.data[y][x / 2] = nelm;
        self
    }
    pub fn tile_at_pixel(&self, pos: VectType) -> MapTile {
        self.pixel_to_index(pos)
            .map_or(MapTile::Empty, |(x, y)| self.get(x, y))
    }
    pub fn pixel_to_index(&self, pos: VectType) -> Option<(usize, usize)> {
        let (x_raw, y_raw) = (pos / TILE_SIZE).get();
        let x_raw = x_raw - BUFFER_TILES;
        let y_raw = y_raw - BUFFER_TILES;
        if x_raw < N::new(0) || y_raw < N::new(0) {
            return None;
        }
        let x = x_raw.trunc() as usize;
        let y = y_raw.trunc() as usize;
        if x >= MAP_WIDTH || y >= MAP_HEIGHT {
            return None;
        }
        Some((x, y))
    }
    pub fn tiles_intersecting(&self, hbox: RectType) -> impl Iterator<Item = MapTile> + '_ {
        let mut poses = Vec::new();

        let corners = [hbox.tl(), hbox.tr(), hbox.bl(), hbox.br()];
        for corner in corners {
            let mc = self.pixel_to_index(corner);
            if poses.contains(&mc) {
                continue;
            }
            poses.push(mc);
        }

        poses
            .into_iter()
            .map(|opt| opt.map_or(MapTile::Empty, |(x, y)| self.get(x, y)))
    }
    pub fn index_to_pixel(&self, (xidx, yidx): (usize, usize)) -> VectType {
        let x = N::from(xidx as i32 + BUFFER_TILES) * TILE_SIZE;
        let y = N::from(yidx as i32 + BUFFER_TILES) * TILE_SIZE;
        VectType::new(x, y)
    }
    pub const fn get(&self, x: usize, y: usize) -> MapTile {
        let elm = self.data[y][x / 2];
        if x % 2 == 0 {
            MapTile::from_u8(elm >> 4)
        } else {
            MapTile::from_u8(elm)
        }
    }
    pub fn flip_all(&mut self) {
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                self.set(x, y, self.get(x, y).flipped())
            }
        }
    }

    pub fn pretty_print(&self) -> String {
        let mut retvl = String::with_capacity(MAP_WIDTH * MAP_HEIGHT + MAP_HEIGHT);
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let tile = self.get(x, y);
                retvl.push(tile.repr());
            }
            retvl.push('\n');
        }
        retvl
    }
}

pub struct GameMap<'a> {
    pub data: BaseMap,
    pub objects: Vec<Object<'a>>,
}

impl<'a> GameMap<'a> {
    pub fn new_undisplayed(data: BaseMap) -> Self {
        Self {
            data,
            objects: Vec::new(),
        }
    }
    pub fn update_display(&mut self, gfx: &'a OamManaged) {
        let mut prev_itr = self.objects.iter_mut();
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tilekind = self.data.get(x, y);
                let Some(tiletag) = tilekind.tag() else {
                    continue;
                };
                let Some(obj) = prev_itr.next() else { continue };

                if !tilekind.can_change() {
                    continue;
                }
                debug_assert_eq!(obj.x(), x as u16 * TILE_SIZE as u16 + TILE_SIZE as u16);
                debug_assert_eq!(obj.y(), y as u16 * TILE_SIZE as u16 + TILE_SIZE as u16);
                obj.set_sprite(gfx.sprite(tiletag.sprite(0)))
                    .set_hflip(tilekind.needs_hflip())
                    .set_vflip(tilekind.needs_vflip());
            }
        }
    }
    pub fn init_display(
        &mut self,
        gfx: &'a OamManaged,
        bg: &mut MapLoan<'_, RegularMap>,
        vram: &mut VRamManager,
    ) {
        self.objects.clear();
        let bg_tiles = &TILEDATA.tiles;
        vram.set_background_palettes(PALETTES);
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tilekind = self.data.get(x, y);
                if let Some(tile_idx) = tilekind.sprite_idx() {
                    bg.set_tile(
                        vram,
                        (x as u16 + 1, y as u16 + 1),
                        bg_tiles,
                        TileSetting::new(tile_idx, false, false, 0),
                    );
                }

                let Some(tiletag) = tilekind.tag() else {
                    continue;
                };
                let mut obj = gfx.object_sprite(tiletag.sprite(0));
                obj.set_position((
                    x as i32 * TILE_SIZE + TILE_SIZE,
                    y as i32 * TILE_SIZE + TILE_SIZE,
                ))
                .set_hflip(tilekind.needs_hflip())
                .set_vflip(tilekind.needs_vflip())
                .show();
                self.objects.push(obj);
            }
        }
    }

    pub fn player_spawns(&self) -> [(usize, usize); 4] {
        self.data.spawns
    }
}
