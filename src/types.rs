#[derive(Debug)]
pub enum Label {
    Load,
    Store,
    Other,
}

#[derive(Debug)]
pub struct Record {
    pub label: Label,
    pub value: u32,
}
