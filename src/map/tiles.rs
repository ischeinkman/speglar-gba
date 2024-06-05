use crate::graphics::tags;
use agb::display::object::Tag;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
#[allow(dead_code)]
pub enum MapTile {
    /// An empty gridspace.
    ///
    /// Repr: 
    #[default]
    Empty = 0,

    /// An unpassable block.
    ///
    /// Repr: x
    Block = 1,

    /// A mirror slanted up.
    ///
    /// Repr: /
    UpMirror = 2,
    /// A mirror slanted down.
    ///
    /// Repr: \
    DownMirror = 3,
    /// A straight-line horizontal mirror, bouncing vertical-direction bullets
    /// straight back but allowing horizontal-direction bullets through.
    ///
    /// Repr: -
    HorizMirror = 4,
    /// A straight-line vertical mirror, bouncing horizontal-direction bullets
    /// straight back but allowing vertical-direction bullets through.
    ///
    /// Repr: |  
    VertMirror = 5,

    /// A horizontal pipe, allowing a bullet through the horizontal direction
    /// but nothing else.
    ///
    /// Repr: =
    HorizPipe = 6,

    /// A vertical pipe, allowing a bullet through the vertical direction but
    /// nothing else.
    ///
    /// Repr: "
    VertPipe = 7,
}

impl From<u8> for MapTile {
    #[inline]
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}

impl MapTile {
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
    pub const fn from_u8(raw: u8) -> Self {
        // SAFETY: All bitvalues of raw & 0x07 are coverred by the enum.
        let masked = raw & 0x07;
        unsafe { core::mem::transmute(masked) }
    }
    pub const fn needs_hflip(self) -> bool {
        matches!(self, MapTile::DownMirror)
    }
    pub const fn needs_vflip(self) -> bool {
        false
    }
    pub const fn can_change(self) -> bool {
        matches!(self, MapTile::DownMirror | MapTile::UpMirror)
    }
    pub const fn flipped(self) -> MapTile {
        match self {
            MapTile::DownMirror => MapTile::UpMirror,
            MapTile::UpMirror => MapTile::DownMirror,
            other => other,
        }
    }
    pub fn sprite_idx(self) -> Option<u16> {
        use MapTile::*;
        match self {
            Empty => None,
            Block => Some(0),
            UpMirror | DownMirror => None,
            HorizMirror => Some(2),
            VertMirror => Some(3),
            HorizPipe => Some(4),
            VertPipe => Some(5),
        }
    }
    pub fn tag(self) -> Option<&'static Tag> {
        use MapTile::*;
        match self {
            UpMirror => Some(tags::MAP_UP_MIRROR),
            DownMirror => Some(tags::MAP_UP_MIRROR),
            _ => None,
        }
    }
    pub const fn allows_player(self) -> bool {
        use MapTile::*;
        matches!(self, Empty | UpMirror | DownMirror | HorizMirror | VertMirror)
    }
    pub const fn repr(self) -> char {
        use MapTile::*;
        match self {
            Empty => ' ', 
            Block => 'x', 
            UpMirror => '/', 
            DownMirror => '\\', 
            HorizMirror => '-',
            VertMirror => '|', 
            HorizPipe => '=', 
            VertPipe => '"',
        }
    }
}
