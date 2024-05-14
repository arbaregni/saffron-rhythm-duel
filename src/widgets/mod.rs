use bevy::prelude::*;
mod lane_box;

pub struct WidgetsPlugin;
impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(lane_box::LaneBoxPlugin)
        ;
    }
}
