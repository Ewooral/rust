//! `crate::garden::vegetables`.

pub const NAME: &str = "asparagus";

#[derive(Debug)]
pub struct Asparagus {
    height_cm: u32,
}

impl Asparagus {
    pub fn new(height_cm: u32) -> Self {
        Self { height_cm }
    }

    pub fn is_ready(&self) -> bool {
        self.height_cm >= 18
    }
}
