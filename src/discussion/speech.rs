use debug_panic::debug_panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const ONE_SECOND: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Speaker {
    pub name: String,
    pub total_speaking_time: Duration,
    pub number_of_speeches_given: u16,
}

impl Speaker {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            total_speaking_time: Duration::from_secs(0),
            number_of_speeches_given: 0,
        }
    }

    fn tick_speaking_time(&mut self) {
        self.total_speaking_time += Duration::from_secs(1);
    }

}

#[derive(Debug)]
pub struct Speech {
    pub speaker: Arc<Mutex<Speaker>>,
    pub duration: Duration,
    pub fcfs_order: usize,
}

impl Speech {
    pub fn tick_clock(&mut self) {
        self.duration += ONE_SECOND;
        match self.speaker.lock() {
            Ok(mut speaker_locked) => speaker_locked.tick_speaking_time(),
            Err(e) => debug_panic!(e.to_string()),
        }
    }
}