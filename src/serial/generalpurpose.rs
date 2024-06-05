
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
    pub enum GpioDirection {
        #[default]
        Input = 0,
        Output = 1,
    }

    pub struct GeneralPurpose<'a> {
        _handle: &'a mut Serial,
    }

    impl<'a> GeneralPurpose<'a> {
        fn initialize() {
            let old = RCNT.read();
            let new = (old & !(1 << 14)) | (1 << 15);
            RCNT.write(new);
        }

        pub fn gpio_config(&self) -> [GpioDirection; 4] {
            let mut retvl = [GpioDirection::Input; 4];
            for idx in 0..retvl.len() {
                let mask = 1 << (4 + idx);
                let value = if RCNT.read() & mask != 0 {
                    GpioDirection::Output
                } else {
                    GpioDirection::Input
                };
                retvl[idx] = value;
            }
            retvl
        }
        pub fn interupt_enabled(&self) -> bool {
            RCNT.read() & (1 << 8) != 0
        }

        pub fn set_interrupt(&self, interupt: bool) {
            let old = RCNT.read();
            let mask = 1 << 8;
            if interupt {
                RCNT.write(old | mask);
            } else {
                RCNT.write(old & !mask);
            }
        }
    }