use bevy::prelude::*;

// Put this component on entities owned by the local user
#[derive(Component)]
#[derive(Debug,Copy,Clone)]
pub struct PlayerMarker;

// Plut this component on entities owned by the remote user
#[derive(Component)]
#[derive(Debug,Copy,Clone)]
pub struct EnemyMarker;

/// Runtime available information on the object's side.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Team {
    /// Local user.
    Player,
    /// Remote user.
    Enemy
}

pub trait Marker : Component + std::fmt::Debug + Clone + Send + Sync + 'static {
    fn team() -> Team;
    fn as_team(&self) -> Team {
        Self::team()
    }
}
impl <T: Marker> From<T> for Team {
    fn from(marker: T) -> Team {
        marker.as_team()
    }
}

impl Marker for PlayerMarker {
    fn team() -> Team {
        Team::Player
    }
}
impl Marker for EnemyMarker {
    fn team() -> Team {
        Team::Enemy
    }
}

pub trait EntityCommandsExt {
    /// Assigns the current entity the marker component corresponding to the specified `team`.
    fn assign_team_marker(&mut self, team: Team) -> &mut Self;
}
impl EntityCommandsExt for bevy::ecs::system::EntityCommands<'_> {
    fn assign_team_marker(&mut self, team: Team) -> &mut Self {
        use Team::*;
        match team {
            Player => self.insert(PlayerMarker),
            Enemy  => self.insert(EnemyMarker)
        }
    }
}
