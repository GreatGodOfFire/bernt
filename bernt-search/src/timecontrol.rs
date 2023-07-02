use std::{time::{Duration, Instant}, sync::atomic::{AtomicBool, Ordering}};

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
            let extra = if t > inc { t - inc } else { 0 };

            // lets hope 25 ms is enough
            let safety_margin = 25;
            let max = if t >= safety_margin {
                t - safety_margin
            } else {
                0
            };

            let x = max.min(inc + extra / limits.movestogo.unwrap_or(30));

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
