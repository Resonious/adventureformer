use std::ops::{Add, Div, Sub, Mul};
use gl::types::*;

#[derive(Copy, Clone)]
pub struct Rect {
    // == low ==
    pub x1: GLfloat,
    pub y1: GLfloat,
    // == high ==
    pub x2: GLfloat,
    pub y2: GLfloat
}

#[derive(Copy, Clone)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T
}

// TODO simd this whole son of a bitch

impl Rect {
    pub fn new(x: GLfloat, y: GLfloat, width: GLfloat, height: GLfloat) -> Rect {
        Rect {
            x1: x, y1: y,
            x2: x + width,
            y2: y + width
        }
    }

    pub fn pos(&self) -> Vec2<GLfloat> {
        Vec2::<GLfloat> { x: self.x1, y: self.y1 }
    }

    pub fn width(&self) -> GLfloat {
        self.x2 - self.x1
    }
    pub fn height(&self) -> GLfloat {
        self.y2 - self.y1
    }
}

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
