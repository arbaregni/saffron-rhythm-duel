use bevy::prelude::*;

use crate::{
    settings::UserSettings,
    CliArgs
};

use crate::lane::{
    Lane,
    LaneMap
};

use super::{
    BBox
};

#[derive(Component)]
pub struct SongPanel {
    bounds: BBox,
    /// Each lane has a bounding box
    lanes: LaneMap<BBox>,
}


impl SongPanel {
    pub fn new(bounds: BBox) -> SongPanel {
        // split the space into 4
        let lanes = bounds.split_horizontal([0.25, 0.25, 0.25, 0.25]);
        let lanes = LaneMap::from(lanes);

        SongPanel {
            bounds,
            lanes
        }

    }

    
    pub fn target_height(&self) -> f32 {
        crate::arrow::Arrow::height()
    }
    pub fn target_line_y(&self) -> f32 {
        self.bounds.bottom() + 0.5 * self.target_height()
    }
    pub fn arrow_drop_line_y(&self) -> f32 {
        // once the arrow is no longer visible, it's too late for the player to click it
        self.bounds.bottom() - self.target_height() * 1.5
    }
    pub fn lane_letter_height(&self) -> f32 {
        // bit more than height of the target
        self.target_height() * 2.5
    }

    /// Returns the bounds of the entire panel
    pub fn bounds(&self) -> &BBox {
        &self.bounds
    }
    pub fn lanes(&self) -> &LaneMap<BBox> {
        &self.lanes
    }
    /// Returns the bounds of a single lane within this panel
    pub fn lane_bounds(&self, lane: Lane) -> &BBox {
        &self.lanes[lane]
    }

    pub fn build<'a, 'w, 's, T>(self,
                        marker: T,
                        commands: &'a mut Commands<'w, 's>,
                        asset_server: &'a AssetServer,
                        settings: &'a UserSettings,
                        cli: &'a CliArgs,
    ) -> SongPanelSetupContext<'a, 'w, 's, T>
        where T: Component + Copy
    {
        SongPanelSetupContext {
            panel: self,
            cli,
            marker,
            commands,
            asset_server,
            settings,
            _extra: (),
        }
    }
}

pub struct SongPanelSetupContext<'a, 'w, 's, T> {
    pub panel: SongPanel,
    pub marker: T,
    pub commands: &'a mut Commands<'w, 's>,
    pub asset_server: &'a AssetServer,
    pub settings: &'a UserSettings,
    pub cli: &'a CliArgs,
    // prevents users from exhaustively pattern matching
    _extra: (),
}


impl <'a, 'w, 's, T> SongPanelSetupContext<'a, 'w, 's, T>
where T: Component + Copy
{

    /// Other modules define methods on the setup context according to their needs


    /// Creates the SongPanel entity itself and drops the setup context object
    pub fn finish(self) {

        let Self {
            commands,
            marker,
            panel,
            ..
        } = self;

        // spawn the panel
        commands.spawn((marker, panel));

    }

}
