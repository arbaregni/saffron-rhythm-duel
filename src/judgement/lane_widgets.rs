use bevy::prelude::*;

use crate::lane::{
    Lane,
};
use crate::layout::{
    Layer,
    SongPanelSetupContext,
};

#[derive(Component)]
pub struct LaneTarget {
    pub lane: Lane,
}
impl LaneTarget {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}

#[derive(Component)]
pub struct LaneLetter {
    pub lane: Lane
}
impl LaneLetter {
    pub fn alpha() -> f32 {
        0.3 // default alpha for the lane letter
    }
}


impl <'a, 'w, 's, T> SongPanelSetupContext<'a, 'w, 's, T>
where T: Component + Copy
{
    /// Creates the targets on the bottom and attaches the appropriate marker
    pub fn setup_lane_targets(self) -> Self {

        for (lane, bounds) in self.panel.lanes().iter() {
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
        for (lane, bounds) in self.panel.lanes().iter() {

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


}
