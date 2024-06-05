pub struct Rng {
    cur_state: u64,
}

impl Rng {
    pub const fn with_seed(seed: u64) -> Self {
        Self { cur_state: seed }
    }
    pub const fn bool_const(self) -> (Self, bool) {
        let next_state = step(self.cur_state);
        (Self::with_seed(next_state), next_state % 2 == 1)
    }
    pub const fn u8_const(self, min: u8, max: u8) -> (Self, u8) {
        let (next, base) = self.u64_const(min as u64, max as u64);
        (next, base as u8)
    }
    pub const fn usize_const(self, min: usize, max: usize) -> (Self, usize) {
        let (next, base) = self.u64_const(min as u64, max as u64);
        (next, base as usize)
    }
    pub const fn u64_const(self, min: u64, max: u64) -> (Self, u64) {
        let next_state = step(self.cur_state);
        let range = max - min + 1;
        (Self::with_seed(next_state), min + next_state % range)
    }
    pub const fn i32_const(self, min: i32, max: i32) -> (Self, i32) {
        let next_state = step(self.cur_state);
        let n = (next_state & ((1 << 31) - 1)) as i32;
        let range = max - min + 1;
        (Self::with_seed(next_state), min + n % range)
    }
}

const fn step(cur: u64) -> u64 {
    let mut retvl = cur;
    retvl ^= retvl << 13;
    retvl ^= retvl >> 7;
    retvl ^= retvl << 17;
    retvl
}
