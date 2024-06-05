use agb::display::object::Object;

use crate::{
    map::{BaseMap, MapTile},
    n_from_bit, Direction, Hitbox, Player, PlayerTag, RectExt, VectType, N,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub enum BulletTag {
    #[default]
    NoPlayer,
    Player1,
    Player2,
    Player3,
    Player4,
}

impl BulletTag {
    pub const fn matches_player(self, player: PlayerTag) -> bool {
        matches!(
            (self, player),
            (BulletTag::Player1, PlayerTag::P1)
                | (BulletTag::Player2, PlayerTag::P2)
                | (BulletTag::Player3, PlayerTag::P3)
                | (BulletTag::Player4, PlayerTag::P4)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub enum BulletType {
    #[default]
    Bullet,
    Reflector,
}

impl BulletTag {
    pub fn hits_player(self, ptag: PlayerTag) -> bool {
        use BulletTag::*;
        use PlayerTag::*;
        !matches!(
            (self, ptag),
            (Player1, P1) | (Player2, P2) | (Player3, P3) | (Player4, P4)
        )
    }
}

pub struct Bullet<'a> {
    pub sprite: Object<'a>,
    pub pos: VectType,
    pub dir: Direction,
    pub tag: BulletTag,
    pub kind: BulletType,
    pub should_die: bool,
}

impl<'a> Hitbox for Bullet<'a> {
    fn pos(&self) -> VectType {
        self.pos
    }
    fn size(&self) -> VectType {
        VectType::new(4.into(), 4.into())
    }
}

impl<'a> Bullet<'a> {
    // Translates to 1.875 pixels per second, based on:
    // * The 5th-from-last bit corresponds to 1/32 pixels per frame
    // * The GBA is 60 FPS
    // * 60 frame/s * 1/32 px/frame = 1.875 px/s
    pub const BULLET_SPEED: N = n_from_bit(5);
    // Translates to 0.9375 pixels per second, based on:
    // * The 5th-from-last bit corresponds to 1/64 pixels per frame
    // * The GBA is 60 FPS
    // * 60 frame/s * 1/64 px/frame = 0.9375 px/s
    pub const SHIELD_SPEED: N = n_from_bit(6);
    const fn speed(&self) -> N {
        match self.kind {
            BulletType::Bullet => Self::BULLET_SPEED,
            BulletType::Reflector => Self::SHIELD_SPEED,
        }
    }
    pub fn update(
        &mut self,
        map: &BaseMap,
        players: &[Player],
        other_bullets_1: &[Bullet],
        other_bullets_2: &[Bullet],
    ) -> Option<BulletEvent> {
        use Direction::*;
        self.pos += self.dir.scaled_vec(self.speed());
        for player in players {
            if !self.collides(player) {
                continue;
            }
            self.should_die = true;
            if self.kind != BulletType::Bullet {
                return None;
            }
            if self.tag.matches_player(player.tag) {
                return Some(BulletEvent::PushChargePlayer(player.tag, self.dir));
            } else {
                return Some(BulletEvent::KillPlayer(player.tag));
            }
        }
        for other in other_bullets_1.iter().chain(other_bullets_2.iter()) {
            if !self.collides(other) {
                continue;
            }
            if self.kind == other.kind {
                self.should_die = true;
                return None;
            }
            if other.kind == BulletType::Reflector {
                self.dir = self.dir.flipped();
            }
        }
        let tile = map.tile_at_pixel(self.hitbox().center());
        match tile {
            MapTile::Block => {
                self.should_die = true;
            }
            MapTile::UpMirror => {
                self.dir = match self.dir {
                    Right => Up,
                    Up => Right,
                    Down => Left,
                    Left => Down,
                };
            }
            MapTile::DownMirror => {
                self.dir = match self.dir {
                    Right => Down,
                    Down => Right,
                    Up => Left,
                    Left => Up,
                };
            }
            MapTile::HorizMirror if self.dir.is_vertical() => {
                self.dir = self.dir.flipped();
            }
            MapTile::HorizMirror => {
                self.should_die = true;
            }
            MapTile::VertMirror if self.dir.is_horizontal() => {
                self.dir = self.dir.flipped();
            }
            MapTile::VertMirror => {
                self.should_die = true;
            }
            MapTile::VertPipe if self.dir.is_horizontal() => {
                self.should_die = true;
            }
            MapTile::HorizPipe if self.dir.is_vertical() => {
                self.should_die = true;
            }
            MapTile::HorizPipe | MapTile::VertPipe | MapTile::Empty => {}
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BulletEvent {
    KillPlayer(PlayerTag),
    PushChargePlayer(PlayerTag, Direction),
}
