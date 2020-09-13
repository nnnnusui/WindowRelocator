#[derive(Debug)]
pub struct Position {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}
impl Position {
    pub fn has_imaginary_size(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}
