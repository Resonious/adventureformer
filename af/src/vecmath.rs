#[derive(Clone)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T
}

impl<T: Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x: x, y: y }
    }
}

impl<T: Copy> Copy for Vec2<T> { }
