pub struct Alu {
    cnt: isize,
}

impl Alu {
    pub fn new() -> Self {
        Alu { cnt: 0 }
    }

    pub fn set(&mut self, value: isize) {
        self.cnt = value;
    }

    pub fn update(&mut self) -> bool {
        let ret = self.cnt;
        self.cnt -= 1;
        if self.cnt < 0 {
            self.cnt = 0;
        }
        ret != 0
    }
}
