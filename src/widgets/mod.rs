use bevy::prelude::*;
mod lane_box;
mod target_sparkles;
mod lane_widgets;

pub struct WidgetsPlugin;
impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
            .add_systems(Update, lane_widgets::darken_on_press)
            .add_systems(Update, lane_widgets::darken_over_time)
            .add_systems(Update, lane_widgets::jostle_on_dropped_note)
            .add_systems(Update, lane_widgets::animate_jostling)
        ;
    }
}
