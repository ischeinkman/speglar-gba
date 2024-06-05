use core::fmt::Debug;

use agb::{
    display::object::{OamManaged, Object, Sprite, Tag},
    fixnum::num,
    input::{Button, ButtonController, Tri},
};
use alloc::vec::Vec;

use crate::{
    map::BaseMap, n_from_parts, AlignedVec, Bullet, BulletTag, Direction, Hitbox, VectType,
    MAX_FRAC_PORTION, N,
};

pub struct Player<'a> {
    pub sprite: Option<Object<'a>>,
    prev_sprite: Option<&'static Sprite>,
    pub pos: VectType,
    pub dir: Direction,
    pub vel: AlignedVec,
    pub charge: u8,
    pub tag: PlayerTag,
}

impl<'a> Debug for Player<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Player<'a>")
            .field("tag", &self.tag)
            .field("pos", &self.pos)
            .field("vel", &self.vel)
            .field("dir", &self.dir)
            .field("charge", &self.charge)
            .field(
                "sprite",
                &(self.sprite.as_ref().map_or("None", |_| "Some(_)")),
            )
            .field(
                "prev_sprite",
                &(self.prev_sprite.as_ref().map_or("None", |_| "Some(_)")),
            )
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum PlayerTag {
    #[default]
    P1 = 0,
    P2 = 1,
    P3 = 2,
    P4 = 3,
}

impl PlayerTag {
    pub const fn from_u8(n: u8) -> Self {
        unsafe { core::mem::transmute(n & 0x3) }
    }
    pub const fn bullet_tag(self) -> BulletTag {
        match self {
            PlayerTag::P1 => BulletTag::Player1,
            PlayerTag::P2 => BulletTag::Player2,
            PlayerTag::P3 => BulletTag::Player3,
            PlayerTag::P4 => BulletTag::Player4,
        }
    }
    pub fn sprite_tag(self) -> &'static Tag {
        crate::graphics::tags::PLAYERS[self as u8 as usize]
    }
}

impl<'a> Hitbox for Player<'a> {
    fn pos(&self) -> VectType {
        self.pos
    }
    fn size(&self) -> VectType {
        VectType::new(num!(7.5), num!(7.5))
    }
}

impl<'a> Player<'a> {
    pub const SPEED: N = n_from_parts(1, 0);
    pub const FRICTION: N = n_from_parts(0, MAX_FRAC_PORTION / 3);
    pub const OVERBOOST_FRICTION: N = n_from_parts(0, MAX_FRAC_PORTION / 2);
    pub const ACCEL: N = n_from_parts(0, MAX_FRAC_PORTION / 2);

    const fn speed_for(dir: Direction) -> AlignedVec {
        AlignedVec::new_unchecked(Self::SPEED, dir)
    }
    pub fn new(pos: VectType, tag: PlayerTag) -> Self {
        let dir = match tag {
            PlayerTag::P1 | PlayerTag::P3 => Direction::Right,
            _ => Direction::Left,
        };

        Self {
            sprite: None,
            prev_sprite: None,
            pos,
            dir,
            vel: AlignedVec::zero(dir),
            charge: 0,
            tag,
        }
    }

    pub fn init_display(&mut self, gfx: &'a OamManaged) {
        self.update_display(gfx);
    }
    pub fn update_display(&mut self, gfx: &'a OamManaged) {
        let mut obj_ref = match self.sprite.take() {
            Some(obj) => obj,
            None => {
                let ret = gfx.object_sprite(self.sprite());
                self.prev_sprite = Some(self.sprite());
                ret
            }
        };
        obj_ref.set_sprite(gfx.sprite(self.sprite()));
        obj_ref
            .set_position(self.pos().trunc())
            .set_hflip(self.hflip())
            .set_vflip(self.vflip())
            .show();
        self.sprite = Some(obj_ref);
    }
    const fn vflip(&self) -> bool {
        matches!(self.dir, Direction::Down)
    }
    const fn hflip(&self) -> bool {
        matches!(self.dir, Direction::Left)
    }
    fn sprite(&self) -> &'static Sprite {
        let tag = self.tag.sprite_tag();
        if self.dir.is_vertical() {
            tag.sprite(0)
        } else {
            tag.sprite(1)
        }
    }
    fn step_vel(&mut self, controls: ControlsRepr) {
        let is_overboost = self.vel.magnitude() > Self::SPEED;
        if is_overboost {
            self.vel = self.vel.step_to(Self::OVERBOOST_FRICTION, num!(0.0));
            self.dir = controls.dir.unwrap_or(self.dir);
        } else {
            match controls.dir {
                None => {
                    self.vel = self.vel.step_to(Self::FRICTION, num!(0.0));
                }
                Some(ndir) => {
                    self.dir = ndir;
                    self.vel = self.vel.step_to_dir(Self::ACCEL, Self::speed_for(self.dir));
                }
            }
        }
    }
    pub fn update(
        &mut self,
        map: &BaseMap,
        players_1: &[Player],
        players_2: &[Player],
        _bullets: &[Bullet],
        controls: ControlsRepr,
    ) {
        self.step_vel(controls);

        let next_pos_raw = self.pos + self.vel;
        let next_pos = {
            let remapped = map.index_to_pixel(map.pixel_to_index(next_pos_raw).unwrap());
            if self.dir.is_horizontal() {
                VectType::new(next_pos_raw.x, remapped.y)
            } else {
                VectType::new(remapped.x, next_pos_raw.y)
            }
        };
        let next_hitbox = self.next_hitbox(next_pos);
        let mut collides = false;
        'outer: {
            let next_tiles = map.tiles_intersecting(next_hitbox).collect::<Vec<_>>();
            for next_tile in next_tiles {
                if !next_tile.allows_player() {
                    collides = true;
                    break 'outer;
                }
            }
            for other in players_1.iter().chain(players_2.iter()) {
                if next_hitbox.collides(other) {
                    collides = true;
                    break 'outer;
                }
            }
        }
        if collides {
            self.vel = AlignedVec::zero(self.dir);
        } else {
            self.pos = next_pos;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ControlsRepr {
    pub dir: Option<Direction>,
    pub fired_bullet: bool,
    pub fired_shield: bool,
}

impl<'a> From<&'a ButtonController> for ControlsRepr {
    fn from(value: &'a ButtonController) -> Self {
        use Direction::*;
        use Tri::*;
        let dir = match (value.y_tri(), value.x_tri()) {
            (Negative, _) => Some(Up),
            (Positive, _) => Some(Down),
            (_, Positive) => Some(Right),
            (_, Negative) => Some(Left),
            _ => None,
        };
        let fired_shield = value.is_just_pressed(Button::A | Button::R);
        let fired_bullet = !fired_shield && value.is_just_pressed(Button::A);
        Self {
            dir,
            fired_bullet,
            fired_shield,
        }
    }
}
