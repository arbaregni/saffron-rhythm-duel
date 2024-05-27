use bevy::prelude::*;

use crate::lane::{
    Lane,
};
use crate::layout::{
    Layer,
    SongPanel,
    SongPanelSetupContext,
};
use crate::input::{
    LaneHit
};
use crate::team_markers::{
    PlayerMarker
};
use crate::judgement::{
    DroppedNoteEvent
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

const DARKENED_DURATION: f32 = 0.25;

#[derive(Component)]
#[component(storage = "SparseSet")] // because we add and remove these often
pub struct DarkeningEffect {
    // timestamp of when this UI object was last triggered
    start_time: f32,
    start_color: Color,
    end_color: Color,
}

pub fn darken_on_press(
    mut commands: Commands,
    time: Res<Time>,
    mut lane_hits: EventReader<LaneHit>,
    lane_targets: Query<(Entity, &LaneTarget), With<PlayerMarker>>,
    lane_letters: Query<(Entity, &LaneLetter), With<PlayerMarker>>,
) {
    let now = time.elapsed().as_secs_f32();

    for lane_hit in lane_hits.read() {
        let event_lane = lane_hit.lane();

        // get the lane targets
        lane_targets
            .iter()
            .filter(|(_, lane_target)| lane_target.lane == event_lane)
            .for_each(|(entity, _)| {
                commands.entity(entity)
                      .insert(DarkeningEffect {
                          start_time: now,
                          start_color: event_lane.colors().heavy,
                          end_color: event_lane.colors().light,
                      });
            });

        // get the lane letters
        lane_letters
            .iter()
            .filter(|(_, lane_letter)| lane_letter.lane == event_lane)
            .for_each(|(entity, _)| {
                commands.entity(entity)
                        .insert(DarkeningEffect {
                            start_time: now,
                            start_color: event_lane.colors().heavy.with_a(LaneLetter::alpha()),
                            end_color: event_lane.colors().light.with_a(LaneLetter::alpha()),
                        });
            });

    }
}

pub fn darken_over_time(
    mut commands: Commands,
    time: Res<Time>,
    mut lane_targets: Query<(Entity, &DarkeningEffect, &mut Sprite), With<LaneTarget>>,
    mut lane_letters: Query<(Entity, &DarkeningEffect, &mut Text), With<LaneLetter>>,
) {
    let now = time.elapsed().as_secs_f32();

    let mut set_color = |id: Entity, effect: &DarkeningEffect, color: &mut Color| {
        let t = (now - effect.start_time) / DARKENED_DURATION;
        *color = effect.start_color * (1.0 - t) + effect.end_color * t;

        if t >= 1.0 {
            // all done: remove the component
            commands.entity(id)
                .remove::<DarkeningEffect>();
        }
    };

    lane_targets
        .iter_mut()
        .for_each(|(id, effect, mut sprite)| {
            set_color(id, effect, &mut sprite.color);
        });

    lane_letters
        .iter_mut()
        .for_each(|(id, effect, mut text)| {
            set_color(id, effect, &mut text.sections[0].style.color);
        });
}

const JOSTLING_DURATION: f32 = 0.3;

#[derive(Component)]
#[component(storage = "SparseSet")] // because they are added and removed
pub struct JostlingEffect {
    start_time: f32,
    pos: Vec3,
    extents: Vec3,
}
pub fn jostle_on_dropped_note(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &LaneLetter, &Transform)>,
    mut dropped_notes: EventReader<DroppedNoteEvent>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
) {
    let now = time.elapsed().as_secs_f32();

    let panel = panel.single();

    for dropped_note in dropped_notes.read() {
        log::debug!("consuming dropped note event");

        let event_lane = dropped_note.arrow().lane();

        let x_extents = panel.lane_bounds(event_lane).width() / 6.0;

        query
            .iter()
            .filter(|(_, lane_letter, _)| lane_letter.lane == event_lane)
            .for_each(|(id, _, transform)| {
                commands.entity(id)
                        .insert(JostlingEffect {
                            start_time: now,
                            pos: transform.translation, // TODO: make this be the center of the
                                                        // lane to allow stacked jostles
                            extents: Vec3::new(x_extents, 0.0, 0.0)
                        });
            });

    }
}
pub fn animate_jostling(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &JostlingEffect, &mut Transform)>,
) {
    let now = time.elapsed().as_secs_f32();

    fn impulse(t: f32) -> f32 {
        use std::f32::consts::PI;

        let freq = 3.0;
        let decay = 1.0 - t;
        (freq * PI * t).sin() * decay
    }

    query
        .iter_mut()
        .for_each(|(id, effect, mut transform)| {
            let t = (now - effect.start_time) / JOSTLING_DURATION;

            let offset = impulse(t) * effect.extents;
            transform.translation = effect.pos + offset;

            if t >= 1.0 {
                transform.translation = effect.pos;
                commands.entity(id)
                        .remove::<JostlingEffect>();
            }
        });

}

