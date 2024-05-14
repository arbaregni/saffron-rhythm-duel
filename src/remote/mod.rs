use bevy::prelude::*;

use crate::{
    CliArgs,
};

fn setup(
    mut _commands: Commands,
    _cli: Res<CliArgs>,
)
{

}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building remote user plugin");
        app
            .add_systems(Startup, setup)
        ;
    }
}
