use std::{
    fmt, cmp, hash
};

use bevy::prelude::*;

// Put this component on entities owned by the local user
#[derive(Component)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct PlayerMarker;

// Plut this component on entities owned by the remote user
#[derive(Component)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct EnemyMarker;

/// Runtime available information on the object's side.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum Team {
    /// Local user.
    Player,
    /// Remote user.
    Enemy
}

pub trait Marker : Component 
    + Clone
    + Send
    + Sync
    + fmt::Debug
    + cmp::Eq
    + hash::Hash
    + 'static
{
    fn marker() -> Self;
    fn team() -> Team;
    fn as_team(&self) -> Team {
        Self::team()
    }
    fn is_local() -> bool {
        match <Self as Marker>::team() {
            Team::Player => true,
            Team::Enemy => false,
        }
    }
}
impl <T: Marker> From<T> for Team {
    fn from(marker: T) -> Team {
        marker.as_team()
    }
}

impl Marker for PlayerMarker {
    fn marker() -> PlayerMarker {
        PlayerMarker{}
    }
    fn team() -> Team {
        Team::Player
    }
}
impl Marker for EnemyMarker {
    fn marker() -> EnemyMarker {
        EnemyMarker{}
    }
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
