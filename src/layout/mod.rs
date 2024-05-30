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
};

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
};

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Default)]
#[derive(States)]
pub enum LayoutState {
    #[default]
    NotReady,
    Done
}

fn setup_layout(
    mut commands: Commands,
    mut state: ResMut<NextState<LayoutState>>,
) {
    let bounds = crate::world();
    let [player_bounds, _, enemy_bounds] = bounds.split_horizontal([0.4, 0.2, 0.4]);

    // create the player song panel
    let player_panel = SongPanel::new(player_bounds);
    commands.spawn((
        PlayerMarker{},
        player_panel
    ));

    let enemy_panel = SongPanel::new(enemy_bounds);
    commands.spawn((
        EnemyMarker{},
        enemy_panel
    ));
    
    // done with laying out, we set this so that now the game objects can spawn in
    state.set(LayoutState::Done);
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<LayoutState>()
            .add_systems(Startup, setup_layout)

            .add_systems(Startup, ui_timer::setup)
            .add_systems(Update, ui_timer::update_time_text);
    }
}
