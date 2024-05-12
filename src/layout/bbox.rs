use bevy::prelude::*;

/// An axis aligned bounding box
#[derive(Debug,Clone)]
pub struct BBox {
    min: Vec3,
    max: Vec3
}

impl BBox {
    pub fn from_edges(left: f32, right: f32, bottom: f32, top: f32) -> BBox {
        let min = Vec3::new(left, bottom, 0.0);
        let max = Vec3::new(right, top, 0.0);
        BBox {
            min, max
        }
    }
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
    pub fn to_rectangle(&self) -> Rectangle {
        Rectangle::new(self.width(), self.height())
    }
    pub fn split_horizontal<const N: usize>(&self, mut split: [f32; N]) -> [BBox; N] {
        let total: f32 = split.iter().copied().sum();

        // normalize what the user gave us.
        if total == 0.0 {
            panic!("splitting bbox, but the total weight was 0.0");
        }
        split.iter_mut()
             .for_each(|x| *x /= total);

        let mut left_margin = 0.0;
        split.map(|x_frac| {
            let width = x_frac * self.width();

            let left = self.left() + left_margin;
            let right = left + width;

            let bottom = self.bottom();
            let top = self.top();

            left_margin += width;

            BBox::from_edges(
                left, right,
                bottom, top
            )
        })
       
    }
}
