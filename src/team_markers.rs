use bevy::prelude::*;

// Put this component on entities owned by the local user
#[derive(Component)]
#[derive(Debug,Copy,Clone)]
pub struct PlayerMarker;

// Plut this component on entities owned by the remote user
#[derive(Component)]
#[derive(Debug,Copy,Clone)]
pub struct EnemyMarker;
