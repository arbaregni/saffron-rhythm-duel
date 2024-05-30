use bevy::prelude::*;
mod lane_box;
mod target_sparkles;
mod lane_widgets;

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
};
use crate::layout::{
    LayoutState
};

pub struct WidgetsPlugin;
impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)

            // everything that needs the song panels to set up gets run here
            .add_systems(OnEnter(LayoutState::Done), (
                    lane_widgets::setup_lane_targets::<PlayerMarker>,
                    lane_widgets::setup_lane_targets::<EnemyMarker>,

                    lane_widgets::setup_lane_letters::<PlayerMarker>,
            ))
            .add_systems(Update, lane_widgets::darken_on_press)
            .add_systems(Update, lane_widgets::darken_over_time)
            .add_systems(Update, lane_widgets::jostle_on_dropped_note)
            .add_systems(Update, lane_widgets::animate_jostling)
        ;
    }
}
