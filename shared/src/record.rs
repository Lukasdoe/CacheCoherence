use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone)]
pub enum Label {
    Load,
    Store,
    Other,
}
