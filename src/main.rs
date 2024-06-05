// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![allow(clippy::assertions_on_constants)]

extern crate alloc;

use agb::{
    display::{
        object::OamManaged,
        tiled::{MapLoan, RegularBackgroundSize, RegularMap, TiledMap, VRamManager},
        Priority,
    },
    input::ButtonController,
    mgba::DebugLevel,
    println, Gba,
};

mod bullet;
mod map;
mod rng;
mod serial;
use alloc::vec::Vec;
use bullet::*;
mod utils;
use map::GameMap;
pub use utils::*;
mod player;
pub use player::*;
mod graphics;

// The main function must take 1 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly. It will also handle creating the `Gba` struct for you.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    main_inner(gba)
}

fn main_inner(mut gba: Gba) -> ! {
    let vblank = agb::interrupt::VBlank::get();
    if let Some(mut mgba) = agb::mgba::Mgba::new() {
        mgba.set_level(DebugLevel::Debug);
    }
    let mut frameidx = 0;
    let test_map = map::generate(0xdeadbeef, map::HONEYCOMB_BASE, 16, 32);
    let gfx = gba.display.object.get_managed();
    let test_map = GameMap::new_undisplayed(test_map);
    let mut game = GameState::new(test_map, PlayerTag::P1);
    let (tiled, mut vram) = gba.display.video.tiled0();
    let mut bg = tiled.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        graphics::TILEDATA.tiles.format(),
    );
    game.init_display(&gfx, &mut bg, &mut vram);
    bg.commit(&mut vram);
    bg.set_visible(true);
    loop {
        game.update_logic();
        vblank.wait_for_vblank();
        game.update_display(&gfx);
        gfx.commit();
        frameidx += 1;
    }
    drop(bg);
    drop(test_map);
}

pub struct GameState<'a> {
    pub map: GameMap<'a>,
    pub players: Vec<Player<'a>>,
    pub bullets: Vec<Bullet<'a>>,
    pub local_player: PlayerTag,
    pub button_controller: ButtonController,
}

impl<'a> GameState<'a> {
    pub fn new(map: GameMap<'a>, local_player: PlayerTag) -> Self {
        let mut players = Vec::with_capacity(4);
        for (pidx, spawn) in map.player_spawns().iter().enumerate() {
            let ptag = PlayerTag::from_u8(pidx as u8);
            let player = Player::new(map.data.index_to_pixel(*spawn), ptag);
            players.push(player);
        }
        Self {
            map,
            players,
            bullets: Vec::new(),
            local_player,
            button_controller: ButtonController::new(),
        }
    }
    pub fn init_display(
        &mut self,
        gfx: &'a OamManaged,
        bg: &mut MapLoan<'_, RegularMap>,
        vram: &mut VRamManager,
    ) {
        self.map.init_display(gfx, bg, vram);
        for player in &mut self.players {
            player.init_display(gfx);
        }
    }
    pub fn update_logic(&mut self) {
        self.button_controller.update();
        for idx in 0..self.players.len() {
            let Some((pa, cur, pb)) = split_mut_at(&mut self.players, idx) else {
                continue;
            };
            let controls = if cur.tag == self.local_player {
                ControlsRepr::from(&self.button_controller)
            } else {
                ControlsRepr::default()
            };

            cur.update(&self.map.data, pa, pb, &self.bullets, controls);
        }
        let mut players_to_remove = Vec::new();
        let mut bullets_to_remove = Vec::new();
        let bullet_n = self.bullets.len();
        for idx in 0..bullet_n {
            let Some((ba, cur, bb)) = split_mut_at(&mut self.bullets, idx) else {
                continue;
            };
            if let Some(evt) = cur.update(&self.map.data, &self.players, ba, bb) {
                match evt {
                    BulletEvent::KillPlayer(tag) => {
                        if let Ok(pidx) = self.players.binary_search_by_key(&tag, |p| p.tag) {
                            players_to_remove.push(pidx);
                        }
                    }
                    other => {
                        println!("TODO: Handle event {:?}", other);
                    }
                }
            }
            if cur.should_die {
                bullets_to_remove.push(idx);
            }
        }
    }
    pub fn update_display(&mut self, gfx: &'a OamManaged) {
        self.map.update_display(gfx);
        for plr in self.players.iter_mut() {
            plr.update_display(gfx);
        }
    }
}
