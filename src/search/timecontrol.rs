use std::time::{Duration, Instant};

use crate::{position::piece::PieceColor, uci::Limits};

pub struct TimeControl {
    pub start: Instant,
    pub stop: Option<Duration>,
}

impl TimeControl {
    pub fn new(time: &Limits, color: PieceColor) -> Self {
        let (t, inc) = match color {
            PieceColor::White => (time.wtime, time.winc),
            PieceColor::Black => (time.btime, time.binc),
        };
        let stop = if let Some(t) = t {
            let inc = inc.unwrap_or(0);
            let extra = if t > inc {
                t - inc
            } else {
                0
            };

            // lets hope 25 ms is enough
            let safety_margin = 25;
            let max = if t >=safety_margin {
                t - safety_margin
            } else {
                0
            };

            let x = max.min(inc + extra / time.movestogo.unwrap_or(30));

            Some(Duration::from_millis(x))
        } else {
            None
        };

        Self {
            start: Instant::now(),
            stop,
        }
    }

    pub fn stop(&self) -> bool {
        if self.stop.is_none() {
            return false;
        }
        self.start.elapsed() > self.stop.unwrap()
    }
}
