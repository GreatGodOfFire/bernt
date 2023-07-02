use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

use bernt_position::Position;
use bernt_search::{Limits, SearchState};

pub struct ThreadHandle {
    pub stop: Arc<AtomicBool>,
    pub searching: Arc<(Mutex<bool>, Condvar)>,
    pub data: Arc<Mutex<(Position, Limits)>>,
}

impl ThreadHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn start_search(&self, position: Position, limits: Limits) {
        self.wait_for_search_finish();
        let mut searching = self.searching.0.lock().unwrap();
        self.stop.store(false, Ordering::Relaxed);
        *searching = true;
        self.data.lock().unwrap().0 = position;
        self.data.lock().unwrap().1 = limits;
        self.searching.1.notify_one();
    }

    #[inline]
    pub fn wait_for_search_finish(&self) {
        let searching = &self.searching.0;
        let _g = self
            .searching
            .1
            .wait_while(searching.lock().unwrap(), |x| *x)
            .unwrap();
    }
}

pub fn start_main() -> ThreadHandle {
    let stop = Arc::new(AtomicBool::new(true));
    let stop2 = Arc::clone(&stop);
    let searching = Arc::new((Mutex::new(false), Condvar::new()));
    let searching2 = Arc::clone(&searching);
    let data = Arc::new(Mutex::new((Position::startpos(), Limits::default())));
    let data2 = Arc::clone(&data);

    let mut state = SearchState::new();

    std::thread::spawn(move || {
        let (searching, cvar) = &*searching2;

        loop {
            {
                let mut searching = searching.lock().unwrap();
                *searching = false;
            }

            let _g = cvar.wait_while(searching.lock().unwrap(), |x| !*x).unwrap();

            state.position = data2.lock().unwrap().0.clone();
            state.limits = data2.lock().unwrap().1.clone();

            println!("bestmove {}", state.search(stop2.as_ref()));
        }
    });

    ThreadHandle {
        stop,
        searching,
        data,
    }
}
