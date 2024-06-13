use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize
};
use crate::keycode_serde;

use crate::song::ArrowSpawner;
use crate::user_settings::UserSettings;
use crate::lane::Lane;
use crate::team_markers::{
    Marker,
    PlayerMarker,
    EnemyMarker,
};

/// Represents a user attempting to complete the note in a lane.
#[derive(Event)]
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct RawLaneHit<T: Marker> {
    /// Lane that was hit
    pub lane: Lane,
    /// When the key was pressed
    pub time_of_hit: f32,
    /// The beat when the key was pressed
    pub beat: f32,
    /// The team (local or remote) that made the hit
    pub _team: T,
}
impl <T: Marker> RawLaneHit<T> {
    pub fn from(lane: Lane, beat: f32, time_of_hit: f32) -> RawLaneHit<T> {
        Self {
            lane,
            time_of_hit,
            beat,
            _team: T::marker()
        }
    }
    pub fn lane(&self) -> Lane {
        self.lane
    }
    pub fn beat(&self) -> f32 {
        self.beat
    }
}

// For convienience
pub type LaneHit = RawLaneHit<PlayerMarker>;
pub type RemoteLaneHit = RawLaneHit<EnemyMarker>;

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
#[allow(non_snake_case)]
pub struct LaneHitControls {
    #[serde(with = "keycode_serde")]
    pub lane_hit_L1: KeyCode,
    #[serde(with = "keycode_serde")]
    pub lane_hit_L2: KeyCode,
    #[serde(with = "keycode_serde")]
    pub lane_hit_R1: KeyCode,
    #[serde(with = "keycode_serde")]
    pub lane_hit_R2: KeyCode,
}
impl std::default::Default for LaneHitControls {
    fn default() -> Self {
        Self {
            lane_hit_L1: KeyCode::KeyA,
            lane_hit_L2: KeyCode::KeyS,
            lane_hit_R1: KeyCode::KeyD,
            lane_hit_R2: KeyCode::KeyF,
        }
    }
}
impl LaneHitControls {
    pub fn keycode(&self, lane: Lane) -> KeyCode {
        match lane {
            Lane::L1 => self.lane_hit_L1,
            Lane::L2 => self.lane_hit_L2,
            Lane::R1 => self.lane_hit_R1,
            Lane::R2 => self.lane_hit_R2,
        }
    }
}

fn listen_for_input(
    time: Res<Time>,
    settings: Res<UserSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    spawner: Query<&ArrowSpawner<PlayerMarker>>,
    mut lane_hit_events: EventWriter<LaneHit>,
) {
    let now = time.elapsed().as_secs_f32();

    let Some(spawner) = spawner.get_single().ok() else {
        return; // nothing to do
    };

    let keymap = &settings.keybindings.lane_hit_keymap;

    Lane::all()
        .iter()
        .map(|&lane| (lane, keymap.keycode(lane)))
        .filter(|(_lane, keycode)| keys.just_pressed(*keycode))
        .map(|(lane, _keycode)| LaneHit {
            lane,
            beat: spawner.curr_beat(),
            time_of_hit: now,
            _team: PlayerMarker{}
        })
        .for_each(|ev| {
            log::debug!("Sending lane hit event");
            lane_hit_events.send(ev);
        });

}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LaneHit>()
            .add_event::<RemoteLaneHit>()
            .add_systems(PreUpdate, listen_for_input) // important that input happens the frame it's detected
        ;
    }
}
