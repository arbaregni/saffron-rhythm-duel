#![allow(dead_code)]

use bevy::prelude::*;

/// An axis aligned bounding box
#[derive(Debug,Clone)]
pub struct BBox {
    min: Vec3,
    max: Vec3
}
impl BBox {
    pub fn from_size(width: f32, height: f32) -> BBox {
        let min = Vec3::new(-width / 2.0, -height / 2.0, 0.0);
        let max = Vec3::new(width / 2.0, height / 2.0, 0.0);
        BBox {
            min, max
        }
    }
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    pub fn center(&self) -> Vec3 {
        self.min / 2.0 + self.max / 2.0
    }
    pub fn extents(&self) -> Vec3 {
        self.size() / 2.0
    }
    pub fn min(&self) -> Vec3 {
        self.min
    }
    pub fn max(&self) -> Vec3 {
        self.max
    }
    pub fn bottom(&self) -> f32 {
        self.min.y
    }
    pub fn top(&self) -> f32 {
        self.max.y
    }
    pub fn left(&self) -> f32 {
        self.min.x
    }
    pub fn right(&self) -> f32 {
        self.max.x
    }
    pub fn width(&self) -> f32 {
        self.size().x
    }
    pub fn height(&self) -> f32 {
        self.size().y
    }
}

#[repr(u8)]
pub enum Layer {
    Arrows = 0,
    Targets = 10,
    AboveTargets = 20,
}
impl Layer {
    // Get the z value of this layer
    pub fn z(self) -> f32 {
        let num = self as u32;
        num as f32
    }
}
