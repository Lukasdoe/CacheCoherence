pub struct Counter {
    pub value: u32,
}

impl Counter {
    pub fn new() -> Self {
        Counter { value: 0 }
    }

    /// Update counter. Returns false if counter reached 0.
    pub fn update(&mut self) -> bool {
        match self.value {
            0 => false,
            c => {
                self.value = c - 1;
                true
            }
        }
    }
}
