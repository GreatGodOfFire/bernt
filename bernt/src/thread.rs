use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

use bernt_position::Position;
use bernt_search::{tt::TranspositionTable, Limits, SearchState};

#[derive(Clone)]
pub struct ThreadHandle {
    pub stop: Arc<AtomicBool>,
    pub searching: Arc<(Mutex<bool>, Condvar)>,
    pub tt: Arc<Mutex<TranspositionTable>>,
    pub data: Arc<Mutex<(Position, Limits)>>,
}

impl ThreadHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn start_search(&self, position: Position, limits: Limits) {
        let mut g = self
            .searching
            .1
            .wait_while(self.searching.0.lock().unwrap(), |x| *x)
            .unwrap();

        self.stop.store(false, Ordering::Relaxed);

        *self.data.lock().unwrap() = (position, limits);

        *g = true;
        self.searching.1.notify_one();
    }
}

pub fn start_main() -> ThreadHandle {
    let stop = Arc::new(AtomicBool::new(true));
    let searching = Arc::new((Mutex::new(false), Condvar::new()));
    let tt = Arc::new(Mutex::new(TranspositionTable::new_default()));
    let data = Arc::new(Mutex::new((Position::startpos(), Limits::default())));

    let this = ThreadHandle {
        stop,
        searching,
        tt,
        data,
    };
    let handle = this.clone();

    let mut state = SearchState::new();

    std::thread::spawn(move || {
        let (searching, cvar) = &*this.searching;

        loop {
            let mut searching = searching.lock().unwrap();
            *searching = false;
            cvar.notify_one();

            searching = cvar.wait_while(searching, |x| !*x).unwrap();
            drop(searching);

            // TODO: Avoid cloning position twice
            let data = this.data.lock().unwrap();
            state.position = data.0.clone();
            state.limits = data.1.clone();
            drop(data);

            println!("bestmove {}", state.search(this.stop.as_ref(), &mut *this.tt.lock().unwrap(), true).unwrap());
        }
    });

    handle
}
