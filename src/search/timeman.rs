use std::time::{Duration, Instant};

use crate::{position::PieceColor, SearchOptions};

use super::consts::{TIMEMAN_HARDDIV, TIMEMAN_SOFTDIV};

pub struct TimeManager {
    pub start: Instant,
    pub hard: Option<Duration>,
    pub soft: Option<Duration>,
}

impl TimeManager {
    pub fn new(options: &SearchOptions, color: PieceColor) -> Self {
        let (t, inc) = match color {
            PieceColor::White => (options.wtime, options.winc),
            PieceColor::Black => (options.btime, options.binc),
        };
        let (hard, soft) = {
            if t == i64::MAX {
                (None, None)
            } else {
                let t = t as f32;
                let inc = inc as f32;
                let extra = (t - inc).max(0.0);

                let max = (t - 25.0).max(0.0);

                let hard = max.min(inc * 3.0 / 4.0 + extra / TIMEMAN_HARDDIV);
                let soft = max.min(inc * 3.0 / 4.0 + extra / TIMEMAN_SOFTDIV);

                (
                    Some(Duration::from_secs_f32(hard / 1000.0)),
                    Some(Duration::from_secs_f32(soft / 1000.0)),
                )
            }
        };

        Self {
            start: Instant::now(),
            hard,
            soft,
        }
    }

    pub fn hard_stop(&self) -> bool {
        if self.hard.is_none() {
            return false;
        }
        self.start.elapsed() > self.hard.unwrap()
    }
    pub fn soft_stop(&self) -> bool {
        if self.soft.is_none() {
            return false;
        }
        self.start.elapsed() > self.soft.unwrap()
    }
}
