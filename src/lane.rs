use bevy::{
    prelude::*,
};

use crate::WORLD_WIDTH;

#[derive(Debug,Copy,Clone)]
#[repr(u8)]
pub enum Lane {
    L1 = 0,
    L2 = 1,
    R1 = 2,
    R2 = 3,
}
impl Lane {
    pub const fn all() -> &'static [Lane] {
        use Lane::*;
        &[L1, L2, R1, R2]
    }
    pub const fn lane_count() -> usize {
        Lane::all().len()
    }
    pub const fn lane_width() -> f32 {
        // to make this function `const`
        let w = WORLD_WIDTH as usize / Lane::lane_count();
        w as f32
    }
    pub fn random() -> Lane {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        Lane::all()
            .choose(&mut rng)
            .copied()
            .expect("at  least one lane")
    }
    pub fn colors(self) -> ColorConfig {
        use Lane::*;
        match self {
            L1 => ColorConfig {
                base: Color::hex("#ff9b71").unwrap(), // Tangerine
                light: Color::rgb_u8(255, 212, 194),
                heavy: Color::rgb_u8(204, 61, 0),
            },
            L2 => ColorConfig {
                base: Color::hex("#a1c084").unwrap(), // Olivine    
                light: Color::rgb_u8(203, 220, 188),
                heavy: Color::rgb_u8(81, 108, 55),
            },
            R1 => ColorConfig {
                base: Color::rgb_u8(153, 153, 255), // Tropical indigo
                light: Color::rgb_u8(204, 204, 255), // periwinkle
                heavy: Color::rgb_u8(71, 71, 255),
            },
            R2 => ColorConfig {
                base: Color::hex("#094d92").unwrap(), // Polynesian blue    
                light: Color::rgb_u8(140, 194, 248),
                heavy: Color::rgb_u8(4, 41, 78),
            },
        }
    }
    pub fn center_x(self) -> f32 {
        let left = -WORLD_WIDTH / 2.0;
        let begin = left + Lane::lane_width() * 0.5;
        let lane_num = self as u8 as f32;

        begin + lane_num * Lane::lane_width()
    }

    pub fn keycode(self) -> KeyCode {
        use Lane::*;
        match self {
            L1 => KeyCode::KeyD,
            L2 => KeyCode::KeyF,
            R1 => KeyCode::KeyJ,
            R2 => KeyCode::KeyK,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColorConfig {
    pub light: Color,
    pub base: Color,
    pub heavy: Color,
}
