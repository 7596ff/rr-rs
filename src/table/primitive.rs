use std::ops::Deref;

#[derive(Debug)]
pub struct Boolean {
    pub result: bool,
}

impl Deref for Boolean {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[derive(Debug)]
pub struct I64 {
    pub result: i64,
}

impl Deref for I64 {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}
