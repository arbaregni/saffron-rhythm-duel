#[repr(u8)]
pub enum Layer {
    Arrows = 0,
    Targets = 10,
    AboveTargets = 20,
    SongEffects = 30,
}
impl Layer {
    // Get the z value of this layer
    pub fn z(self) -> f32 {
        let num = self as u32;
        num as f32
    }
}