use debug_panic::debug_panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashSet;

const ZERO_SECONDS: Duration = Duration::from_secs(0);
const ONE_SECOND: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct Speaker {
    pub name: String,
    pub aliases: HashSet<String>,
    pub total_speaking_time: Duration,
    pub number_of_speeches_given: u16,
}

impl Speaker {

    pub fn new(name: String) -> Self {
        Self {
            name: name,
            aliases: HashSet::new(),
            total_speaking_time: ZERO_SECONDS,
            number_of_speeches_given: 0,
        }
    }

    pub fn merge_with(&mut self, other: Self) {
        for alias in other.aliases {
            self.aliases.insert(alias);
        }
        self.aliases.insert(other.name);
        self.total_speaking_time += other.total_speaking_time;
        self.number_of_speeches_given += other.number_of_speeches_given;
    }
    
    /*
    pub fn has_the_same_name_as(&self, other: Self) -> Option<&String> {

        if self.name == other.name || other.aliases.contains(&self.name) {
            return Some(&self.name);
        }

        for alias in &self.aliases {
            if *alias == other.name || other.aliases.contains(alias) {
                return Some(alias);
            }
        }

        return None;
    }
    */

    fn tick_speaking_time(&mut self) {
        self.total_speaking_time += ONE_SECOND;
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