use unicase::UniCase;
use std::sync::{Arc, Mutex};
use std::time::Duration;


#[derive(Debug, PartialEq, Eq)]
pub struct Speaker {
    pub name: UniCase<String>,
    pub total_speaking_time: Duration,
    pub number_of_speeches_given: u16,
}

impl Speaker {

    pub fn new(name: impl Into<UniCase<String>>) -> Self {
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
    pub is_response: bool,
    pub fcfs_order: usize,
}

impl Speech {

    pub fn new(speaker: Arc<Mutex<Speaker>>, is_response: bool, fcfs_order: usize) -> Self {
        speaker.lock().unwrap().number_of_speeches_given += 1;
        return Self {
            speaker: speaker,
            duration: Duration::from_secs(0),
            is_response: is_response,
            fcfs_order: fcfs_order,
        };
    }

    pub fn tick_clock(&mut self) {
        self.duration += Duration::from_secs(1);
        self.speaker.lock().unwrap().tick_speaking_time();
    }

}