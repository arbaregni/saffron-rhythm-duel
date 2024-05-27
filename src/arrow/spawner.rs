use bevy::prelude::*;

use crate::team_markers::{
    Team,
};

use super::{
    chart::Chart,
    arrow::Arrow
};

#[derive(Component)]
#[derive(Debug, Clone)]
pub struct ArrowSpawner {
    /// How we will spawn the arrows
    chart: Option<Chart>,
    /// The team we are spawning for.
    team: Team,
}
impl ArrowSpawner {
    /// Creates an arrow spawner
    pub fn create(team: Team) -> Self {
        Self {
            chart: None,
            team,
        }
    }

    pub fn chart(&self) -> Option<&Chart> {
        self.chart.as_ref()
    }
    pub fn team(&self) -> Team {
        self.team
    }

    /// Populates `buf` with a list of chart names that the user can select.
    /// Returns Ok or a user friendly error.
    pub fn selectable_chart_names(buf: &mut Vec<String>) -> Result<(), String> {
        use std::fs;
        let parent_path = "assets/charts/";
        
        let paths = fs::read_dir(parent_path)
            .map_err(|e| {
                format!("Unable to read available chart names from `assets/charts/`: {e}")
            })?;

        buf.clear();
        for path in paths {
            let path = path
                .map(|p| p.path().display().to_string())
                .unwrap_or("<unable to read>".to_string());
            buf.push(path);
        }

        Ok(())
    }

    pub fn set_chart(&mut self, chart: Chart) {
        self.chart = Some(chart);
    }

    /// Creates the arrows ands appends them to the given buffer
    pub fn create_arrows_in(&self, buf: &mut Vec<Arrow>, time: &Time, beat: u32) {
        let now = time.elapsed().as_secs_f32();

        let Some(chart) = self.chart.as_ref() else {
            return;
        };

        let lead_time = chart.lead_time_secs();
        for note in chart.get(beat) {
            let lane = note.lane();
            let arrow = Arrow::new(lane, now, now + lead_time, beat);
            buf.push(arrow);
        }
    }

}


#[derive(Component)]
/// Component that holds scratch space for spawning arrows
pub struct ArrowBuf {
    pub buf: Vec<Arrow>
}
impl ArrowBuf {
    pub fn new() -> Self {
        Self {
            // capacity for 4 arrows because we will have at most 1 per lane
            buf: Vec::with_capacity(4)
        }
    }
}




