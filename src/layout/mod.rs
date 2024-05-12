#![allow(dead_code)]

use bevy::prelude::*;

mod bbox;
pub use bbox::{
    BBox
};

mod layers;
pub use layers::{
    Layer
};

mod ui_timer;
mod song_panel;
pub use song_panel::{
    SongPanel,
    SongPanelSetupContext,
};

use crate::{
    Config
};
use crate::team_markers::{
    PlayerMarker
};

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    let bounds = crate::world();
    let [player_bounds, _, _enemy_bounds] = bounds.split_horizontal([0.4, 0.2, 0.4]);

    // create the player song panel
    SongPanel::new(player_bounds)
        .build(PlayerMarker, 
               &mut commands,
               asset_server.as_ref(),
               config.as_ref(),
        )
        // creates all the associated resources
        .setup_lane_targets()
        .setup_feedback_text()
        .setup_lane_letters()
        .finish();

    // TODO: create the enemy song panel
    // SongPanel::new(enemy_bounds);



}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui)
            .add_systems(Startup, ui_timer::setup)
            .add_systems(Update, ui_timer::update_time_text);
    }
}
