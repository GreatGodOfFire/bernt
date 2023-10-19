use std::time::{Duration, Instant};

use crate::{position::PieceColor, SearchOptions};

pub struct TimeManager {
    pub start: Instant,
    pub max: Option<Duration>,
}

impl TimeManager {
    pub fn new(options: &SearchOptions, color: PieceColor) -> Self {
        let (t, inc) = match color {
            PieceColor::White => (options.wtime, options.winc),
            PieceColor::Black => (options.btime, options.binc),
        };
        let max = {
            if t == u64::MAX {
                None
            } else {
                let t = t as f32;
                let inc = inc as f32;
                let extra = (t - inc).max(0.0);

                let max = (t - 25.0).max(0.0);

                let x = max.min(inc * 3.0 / 4.0 + extra / 30 as f32);

                Some(Duration::from_secs_f32(x / 1000.0))
            }
        };

        Self {
            start: Instant::now(),
            max,
        }
    }

    pub fn stop(&self) -> bool {
        if self.max.is_none() {
            return false;
        }
        self.start.elapsed() > self.max.unwrap()
    }
}
