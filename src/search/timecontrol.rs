use std::time::{Duration, Instant};

use crate::{position::piece::PieceColor, uci::Limits};

pub struct TimeControl {
    pub start: Instant,
    pub stop: Option<Duration>,
}

impl TimeControl {
    pub fn new(time: &Limits, color: PieceColor) -> Self {
        let t = match color {
            PieceColor::White => time.wtime,
            PieceColor::Black => time.btime,
        };
        let stop = if let Some(t) = t {
            let x = t / time.movestogo.unwrap_or(40) / 10 * 9;
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
