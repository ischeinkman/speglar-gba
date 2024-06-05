use core::mem;

use super::*;

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default)]
pub enum PlayerId {
    #[default]
    Parent = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

pub struct MultiplayerSerial<'a> {
    _handle: &'a mut Serial,
    is_parent: bool,
    playerid: Option<PlayerId>,
    rate: BaudRate,
}

#[allow(dead_code)]
impl<'a> MultiplayerSerial<'a> {
    pub fn new(_handle: &'a mut Serial, rate: BaudRate) -> Result<Self, InitializationError> {
       let mut retvl = Self {
            _handle,
            is_parent : false,
            playerid: None,
            rate,
        };
        retvl.initialize()?;
        Ok(retvl)
    }

    fn initialize(&mut self) -> Result<(), InitializationError> {
        // FROM https://rust-console.github.io/gbatek-gbaonly/#siomultiplayermode:
        let rcnt = RcntWrapper::new();
        let siocnt = MultiplayerSiocnt::get();

        rcnt.set_mode(SerialMode::Multiplayer);
        siocnt.set_mode(SerialMode::Multiplayer);
        siocnt.set_baud_rate(self.rate);

        let is_okay = siocnt.reg.read_bit(3);
        if !is_okay {
            return Err(InitializationError::FailedOkayCheck);
        }
        let is_parent = siocnt.reg.read_bit(2);
        self.is_parent = is_parent;
        self.playerid = self.is_parent.then_some(PlayerId::Parent);
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InitializationError {
    FailedOkayCheck,
}

pub struct MultiplayerSiocnt {
    inner: SiocntWrapper,
}

method_wraps!(MultiplayerSiocnt, inner, SiocntWrapper);

/*
  Bit   Expl.
  0-1   Baud Rate     (0-3: 9600,38400,57600,115200 bps)
  2     SI-Terminal   (0=Parent, 1=Child)                  (Read Only)
  3     SD-Terminal   (0=Bad connection, 1=All GBAs Ready) (Read Only)
  4-5   Multi-Player ID     (0=Parent, 1-3=1st-3rd child)  (Read Only)
  6     Multi-Player Error  (0=Normal, 1=Error)            (Read Only)
  7     Start/Busy Bit      (0=Inactive, 1=Start/Busy) (Read Only for Slaves)
  8-11  Not used            (R/W, should be 0)
  12    Must be "0" for Multi-Player mode
  13    Must be "1" for Multi-Player mode
  14    IRQ Enable          (0=Disable, 1=Want IRQ upon completion)
  15    Not used            (Read only, always 0)
*/
impl MultiplayerSiocnt {
    const fn new() -> Self {
        Self {
            inner: SiocntWrapper::new(),
        }
    }
    pub const fn get() -> Self {
        Self::new()
    }
    pub fn baud_rate(&self) -> BaudRate {
        let v = self.read();
        let bits = (v & 3) as u8;
        unsafe { core::mem::transmute(bits) }
    }

    pub fn set_baud_rate(&self, rate: BaudRate) {
        let old = self.read();
        let new = (old & !3) | rate as u16;
        self.write(new)
    }

    pub fn is_parent(&self) -> bool {
        self.read_bit(2)
    }

    pub fn gbas_ready(&self) -> bool {
        self.read_bit(3)
    }

    pub fn id(&self) -> PlayerId {
        let regval = self.read();
        let raw = ((regval & (3 << 4)) >> 4) as u8;
        unsafe { mem::transmute(raw) }
    }

    pub fn error_flag(&self) -> bool {
        self.read_bit(6)
    }

    pub fn start_transfer(&self) {
        self.write_bit(7, true)
    }
    pub fn busy(&self) -> bool {
        self.read_bit(7)
    }
}

pub struct MultiplayerCommReg {
    player_id: PlayerId,
    reg: VolAddress<u16, Safe, Safe>,
}

impl MultiplayerCommReg {
    pub const PARENT: Self = MultiplayerCommReg::new(PlayerId::Parent);
    pub const P1: Self = MultiplayerCommReg::new(PlayerId::P1);
    pub const P2: Self = MultiplayerCommReg::new(PlayerId::P2);
    pub const P3: Self = MultiplayerCommReg::new(PlayerId::P3);
    pub const ALL: [Self; 4] = [Self::PARENT, Self::P1, Self::P2, Self::P3];
    pub const fn new(player_id: PlayerId) -> Self {
        let addr = match player_id {
            PlayerId::Parent => 0x4000120,
            PlayerId::P1 => 0x4000122,
            PlayerId::P2 => 0x4000124,
            PlayerId::P3 => 0x4000126,
        };
        let reg = unsafe { VolAddress::new(addr) };
        Self { player_id, reg }
    }

    pub fn read(&self) -> Option<u16> {
        let raw = self.raw_read();
        if raw == 0xFFFF {
            None
        } else {
            Some(raw)
        }
    }
    pub fn raw_read(&self) -> u16 {
        self.reg.read()
    }
    pub fn is_transfering(&self) -> bool {
        self.raw_read() == 0xFFFF
    }
}
