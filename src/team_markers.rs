use std::{
    fmt, cmp, hash
};

use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize
};

#[derive(Component)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash,Deserialize,Serialize)]
#[derive(Reflect)]
/// Put this component on entities owned by the local user
pub struct PlayerMarker;

#[derive(Component)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash,Deserialize,Serialize)]
#[derive(Reflect)]
/// Put this component on entities owned by the remote user
pub struct EnemyMarker;

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash,Deserialize,Serialize)]
#[derive(Reflect)]
/// Runtime available information on the object's side.
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
    + bevy_reflect::FromReflect
    + bevy_reflect::TypePath
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
    fn is_remote() -> bool {
        match Self::team() {
            Team::Enemy => true,
            Team::Player => false,
        }
    }
    fn as_str() -> &'static str {
        match Self::team() {
            Team::Enemy => "enemy",
            Team::Player => "player",
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
