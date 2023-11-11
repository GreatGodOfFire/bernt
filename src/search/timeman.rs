use std::time::{Duration, Instant};

use crate::{position::PieceColor, SearchOptions};

use super::consts::SearchConsts;

pub struct TimeManager {
    pub start: Instant,
    pub hard: Option<Duration>,
    pub soft: Option<Duration>,
}

impl TimeManager {
    pub fn new(options: &SearchOptions, consts: &SearchConsts, color: PieceColor) -> Self {
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

                let max = (t - 25.0).max(0.0);

                let hard = max.min(inc * 3.0 / 4.0 + t / consts.time_harddiv);
                let soft = max.min(inc * 3.0 / 4.0 + t / consts.time_softdiv);

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
