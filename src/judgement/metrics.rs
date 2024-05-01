use bevy::prelude::*;

use super::{
    CorrectHitEvent,
    MissfireEvent,
    Judgement,
};

#[derive(Resource)]
pub struct SongMetrics {
    /// Total number of arrows that have passed the target line.
    pub (in crate::judgement) total_arrows: u32,
    /// Number of arrows that the user has correctly intercepted in time.
    pub (in crate::judgement) success_arrows: u32,
    /// Number of consecutive arrows the user has gotten correct.
    pub (in crate::judgement) streak: u32
}

impl SongMetrics {
    pub fn new() -> SongMetrics {
        SongMetrics {
            total_arrows: 0,
            success_arrows: 0,
            streak: 0,
        }
    }
}

fn update_metrics(
    mut metrics: ResMut<SongMetrics>,
    mut hit_events: EventReader<CorrectHitEvent>,
    mut missfire_events: EventReader<MissfireEvent>
) {
    for hit_ev in hit_events.read() {
        metrics.total_arrows += 1;

        match hit_ev.judgement {
            Judgement::Success => {
                metrics.success_arrows += 1;
                metrics.streak += 1;
            }
            Judgement::Failure => {
                metrics.streak = 0;
            }
        }
    }

    for _missfire_ev in missfire_events.read() {
        metrics.total_arrows += 1;
        metrics.streak = 0;
    }

}

pub struct MetricsPlugin;
impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building metrics plugin");
        app
            .insert_resource(SongMetrics::new())
            .add_systems(Update, update_metrics)
        ;
    }
}
