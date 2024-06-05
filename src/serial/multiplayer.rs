use core::{
    cell::UnsafeCell,
    mem, ptr,
    sync::atomic::{AtomicI32, Ordering},
};

use agb::{
    external::critical_section::CriticalSection,
    interrupt::{add_interrupt_handler, Interrupt, InterruptHandler},
};

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
    _interrupt: InterruptHandler,
    siocnt: MultiplayerSiocnt,
    is_parent: bool,
    playerid: Option<PlayerId>,
    rate: BaudRate,
}

impl<'a> MultiplayerSerial<'a> {
    fn new(_handle: &'a mut Serial, rate: BaudRate) -> Result<Self, InitializationError> {
        // FROM https://rust-console.github.io/gbatek-gbaonly/#siomultiplayermode:
        let rcnt = RcntWrapper::new();
        let siocnt = MultiplayerSiocnt::new();

        rcnt.set_mode(SerialMode::Multiplayer);
        siocnt.set_mode(SerialMode::Multiplayer);
        siocnt.set_baud_rate(rate);

        let is_okay = siocnt.reg.read_bit(3);
        if !is_okay {
            return Err(InitializationError::FailedOkayCheck);
        }
        let is_parent = siocnt.reg.read_bit(2);
        let handler = unsafe { add_interrupt_handler(Interrupt::Serial, _on_irq) };
        Ok(Self {
            _handle,
            _interrupt: handler,
            siocnt,
            is_parent,
            playerid: None,
            rate,
        })
    }

    fn wait_for_send(&self) {
        let old_count = _get_irq_count();
        if self.is_parent {
            let old = SIOCNT.read();
            let new = old | 1 << 7;
            SIOCNT.write(new);
        }
        while _get_irq_count() == old_count {}
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InitializationError {
    FailedOkayCheck,
}

static mut COUNTER: UnsafeCell<u32> = UnsafeCell::new(0);
fn _on_irq(c: CriticalSection<'_>) {
    unsafe {
        let old: u32 = ptr::read_volatile(COUNTER.get() as *const _);
        let new = old.wrapping_add(1);
        ptr::write_volatile(COUNTER.get(), new);
    }
}
fn _get_irq_count() -> u32 {
    unsafe { ptr::read_volatile(COUNTER.get() as *const _) }
}

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

pub struct MultiplayerSiocnt {
    inner: SiocntWrapper,
}

impl AsRef<SiocntWrapper> for MultiplayerSiocnt {
    fn as_ref(&self) -> &SiocntWrapper {
        &self.inner
    }
}
impl AsMut<SiocntWrapper> for MultiplayerSiocnt {
    fn as_mut(&mut self) -> &mut SiocntWrapper {
        &mut self.inner
    }
}

impl Deref for MultiplayerSiocnt {
    type Target = SiocntWrapper;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for MultiplayerSiocnt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl MultiplayerSiocnt {
    pub const fn new() -> Self {
        Self {
            inner: SiocntWrapper::new(),
        }
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
