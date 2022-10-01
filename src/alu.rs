pub struct Alu {
    cnt: u32,
}

impl Alu {
    pub fn new() -> Self {
        Alu { cnt: 0 }
    }

    pub fn set(&mut self, value: u32) {
        self.cnt = value;
    }

    pub fn get(&self) -> u32 {
        self.cnt
    }

    pub fn increase(&mut self, value: u32) {
        self.cnt += value;
    }

    pub fn update(&mut self) -> bool {
        match self.cnt {
            0 => false,
            c => {
                self.cnt = c - 1;
                true
            }
        }
    }
}
