use std::ops::{Add, Div, Sub, Mul};

#[derive(Copy, Clone)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T
}

// TODO simd this son of a bitch
impl<T: Add + Copy> Add for Vec2<T> where T::Output: Copy {
    type Output = Vec2<T::Output>;

    fn add(self, rhs: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}
impl<T: Sub + Copy> Sub for Vec2<T> where T::Output: Copy {
    type Output = Vec2<T::Output>;

    fn sub(self, rhs: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}
impl<T: Mul + Copy> Mul for Vec2<T> where T::Output: Copy {
    type Output = Vec2<T::Output>;

    fn mul(self, rhs: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x * rhs.x,
            y: self.y * rhs.y
        }
    }
}
impl<T: Div + Copy> Div for Vec2<T> where T::Output: Copy {
    type Output = Vec2<T::Output>;

    fn div(self, rhs: Vec2<T>) -> Vec2<T::Output> {
        Vec2::<T::Output> {
            x: self.x / rhs.x,
            y: self.y / rhs.y
        }
    }
}

impl<T: Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x: x, y: y }
    }
    pub fn s(s: T) -> Vec2<T> {
        Vec2 { x: s, y: s }
    }
}
