#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HalfMoveClock {
    clock: u8,
}

impl HalfMoveClock {
    pub fn new() -> HalfMoveClock {
        HalfMoveClock { clock: 0 }
    }

    pub fn get(&self) -> u8 {
        self.clock
    }

    pub fn new_from_clock(clock: u8) -> HalfMoveClock {
        HalfMoveClock { clock }
    }

    pub fn reset(&mut self) {
        self.clock = 0;
    }

    pub fn advance(&mut self) {
        self.clock += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FullMoveCounter {
    counter: u16,
}

impl FullMoveCounter {
    pub fn new() -> FullMoveCounter {
        FullMoveCounter { counter: 1 }
    }

    pub fn get(&self) -> u16 {
        self.counter
    }

    pub fn new_from_counter(counter: u16) -> FullMoveCounter {
        FullMoveCounter { counter }
    }

    pub fn inc(&mut self) {
        self.counter += 1;
    }
}
