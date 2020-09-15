#[derive(Clone, Debug)]
pub struct Position {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}
impl Position {
    pub fn end_point(&self) -> (i32, i32) {
        (self.x + self.width, self.y + self.height)
    }
    pub fn has_imaginary_size(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
    pub fn can_hold(&self, other: &Self) -> bool {
        let self_end = self.end_point();
        let other_end = self.end_point();
        self.x <= other.x
            || self.y <= other.y
            || self_end.0 >= other_end.0
            || self_end.1 >= other_end.1
    }
}
