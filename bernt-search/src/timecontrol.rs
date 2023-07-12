use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use bernt_position::piece::PieceColor;

use crate::Limits;

pub struct TimeControl<'a> {
    pub start: Instant,
    pub max: Option<Duration>,
    pub stop: &'a AtomicBool,
}

impl<'a> TimeControl<'a> {
    pub fn new(limits: &Limits, color: PieceColor, stop: &'a AtomicBool) -> Self {
        let (t, inc) = match color {
            PieceColor::White => (limits.wtime, limits.winc),
            PieceColor::Black => (limits.btime, limits.binc),
        };
        let max = if let Some(t) = t {
            let inc = inc.unwrap_or(0);
            let extra = t.wrapping_sub(inc);

            // lets hope 10 ms is enough
            let max = t.wrapping_sub(10);

            let x = max.min(inc * 3 / 4 + extra / limits.movestogo.unwrap_or(30));

            Some(Duration::from_millis(x))
        } else {
            None
        };

        Self {
            start: Instant::now(),
            max,
            stop,
        }
    }

    pub fn stop(&self) -> bool {
        if self.max.is_none() {
            return self.stop.load(Ordering::Relaxed);
        }
        (self.start.elapsed() > self.max.unwrap()) || self.stop.load(Ordering::Relaxed)
    }
}
