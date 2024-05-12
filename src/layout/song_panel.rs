use bevy::prelude::*;

use crate::{
    Config
};

use crate::lane::{
    Lane,
    LaneMap
};
use crate::judgement::{
    LaneTarget,
    LaneLetter,
};

use super::{
    Layer,
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
    /// Returns the bounds of a single lane within this panel
    pub fn lane_bounds(&self, lane: Lane) -> &BBox {
        &self.lanes[lane]
    }

    pub fn build<'a, 'w, 's, T>(self,
                        marker: T,
                        commands: &'a mut Commands<'w, 's>,
                        asset_server: &'a AssetServer,
                        config: &'a Config,
    ) -> SongPanelSetupContext<'a, 'w, 's, T>
        where T: Component + Copy
    {
        SongPanelSetupContext {
            panel: self,
            marker,
            commands,
            asset_server,
            config,
            _extra: (),
        }
    }
}

pub struct SongPanelSetupContext<'a, 'w, 's, T> {
    pub panel: SongPanel,
    pub marker: T,
    pub commands: &'a mut Commands<'w, 's>,
    pub asset_server: &'a AssetServer,
    pub config: &'a Config,
    // prevents users from exhaustively pattern matching
    _extra: (),
}


impl <'a, 'w, 's, T> SongPanelSetupContext<'a, 'w, 's, T>
where T: Component + Copy
{

    /// Creates the targets on the bottom and attaches the appropriate marker
    pub fn setup_lane_targets(self) -> Self {

        for (lane, bounds) in self.panel.lanes.iter() {
            let lane_target = LaneTarget {
                lane
            };

            let x = bounds.center().x;
            let y = self.panel.target_line_y();
            let z = Layer::Targets.z();
            let pos = Vec3::new(x, y, z);

            let width = bounds.width();
            let height = self.panel.target_height();
            let scale = Vec3::new(width, height, 1.0);

            let transform = Transform {
                translation: pos,
                scale,
                ..default()
            };

            let color = lane.colors().light;
            let sprite = Sprite {
                color,
                ..default()
            };

            self.commands
                .spawn((
                    self.marker,
                    lane_target,
                    SpriteBundle {
                        transform,
                        sprite,
                        ..default()
                    }
                ));

        }

        self
    }

    /// Creates the letters on the bottom and attaches the appropriate marker
    pub fn setup_lane_letters(self) -> Self {
        for (lane, bounds) in self.panel.lanes.iter() {

            let text_content = self.config.keybindings.key_name(lane).to_uppercase();

            let font = self.asset_server.load(crate::BASE_FONT_NAME);
            let font_size = 50.0;
            let color = lane.colors().light.with_a(LaneLetter::alpha());
            
            let x = bounds.center().x;
            let y = self.panel.target_line_y() + self.panel.lane_letter_height();
            let z = Layer::AboveTargets.z();

            let transform = Transform {
                translation: Vec3::new(x, y, z),
                ..default()
            };

            let style = TextStyle { font, font_size, color };
            let text = Text {
                sections: vec![
                    TextSection {
                        value: text_content,
                        style,
                    }
                ],
                ..default()
            };

            self.commands.spawn((
                self.marker,
                LaneLetter {
                    lane
                },
                Text2dBundle {
                    text,
                    transform,
                    ..default()
                }
            ));
            
        }
             
        self
    }


    /// Creates the SongPanel entity and drops the setup context object
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
