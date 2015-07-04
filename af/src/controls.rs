extern crate glfw;

use std::mem::{transmute, size_of};
use std::slice;
use std::slice::IterMut;

pub struct Control {
    pub last_frame: bool,
    pub this_frame: bool
}

impl Control {
    pub fn down(&self)      -> bool { self.this_frame }
    pub fn up(&self)        -> bool { !self.this_frame }
    pub fn just_down(&self) -> bool { self.this_frame && !self.last_frame }
    pub fn just_up(&self)   -> bool { !self.this_frame && self.last_frame }
}

pub struct Controls {
    pub up: Control,
    pub down: Control,
    pub left: Control,
    pub right: Control,
    pub debug: Control,
}

impl Controls {
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Control> {
        unsafe {
            let len = size_of::<Controls>() / size_of::<Control>();
            let mut ctrls_slice: &mut [Control] =
                slice::from_raw_parts_mut(transmute(self), len);
            ctrls_slice.iter_mut()
        }
    }
}
