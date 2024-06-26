use serde::{
    Serialize,
    Deserialize,
};

use bevy::prelude::*;

#[derive(Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[derive(Reflect)]
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
    pub fn colors(self) -> ColorConfig {
        use Lane::*;
        match self {
            L1 => ColorConfig {
                base: Color::hex("#ff9b71").unwrap(), // Tangerine
                light: Color::rgb_u8(255, 212, 194),
                heavy: Color::rgb_u8(204, 61, 0),
                greyed: Color::rgb_u8(120, 93, 81),
            },
            L2 => ColorConfig {
                base: Color::hex("#a1c084").unwrap(), // Olivine    
                light: Color::rgb_u8(203, 220, 188),
                heavy: Color::rgb_u8(81, 108, 55),
                greyed: Color::rgb_u8(97, 112, 82),
            },
            R1 => ColorConfig {
                base: Color::rgb_u8(153, 153, 255), // Tropical indigo
                light: Color::rgb_u8(204, 204, 255), // periwinkle
                heavy: Color::rgb_u8(71, 71, 255),
                greyed: Color::rgb_u8(109, 109, 158),
            },
            R2 => ColorConfig {
                base: Color::hex("#094d92").unwrap(), // Polynesian blue    
                light: Color::rgb_u8(140, 194, 248),
                heavy: Color::rgb_u8(4, 41, 78),
                greyed: Color::rgb_u8(48, 82, 117),
            },
        }
    }
    pub fn as_str(self) -> &'static str {
        use Lane::*;
        match self {
            L1 => "L1",
            L2 => "L2",
            R1 => "R1",
            R2 => "R2",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColorConfig {
    pub light: Color,
    pub base: Color,
    pub heavy: Color,
    #[allow(dead_code)]
    pub greyed: Color,
}

#[derive(Debug, Clone)]
pub struct LaneMap<T> {
    items: [T; 4]
}
impl <T: Default> LaneMap<T> {
    #[allow(dead_code)]
    pub fn new() -> LaneMap<T> {
        LaneMap {
            items: [T::default(), T::default(), T::default(), T::default()]
        }
    }
}
impl <T> LaneMap<T> {
    pub fn from(items: [T; 4]) -> LaneMap<T> {
        LaneMap {
            items
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (Lane, &T)> {
        Lane::all()
            .iter()
            .map(|lane| (*lane, &self[*lane]))
    }
}

impl <T> std::ops::Index<Lane> for LaneMap<T> {
    type Output = T;
    fn index(&self, lane: Lane) -> &T {
        let idx = lane as u8 as usize;
        &self.items[idx]
    }
}
impl <T> std::ops::IndexMut<Lane> for LaneMap<T> {
    fn index_mut(&mut self, lane: Lane) -> &mut T {
        let idx = lane as u8 as usize;
        &mut self.items[idx]
    }
}

