
use std::sync::{Arc, Mutex};
use std::time::Duration;
use unicase::UniCase;
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use std::collections::LinkedList;

use self::speech::{Speaker, Speech};

mod speech;

pub enum PriorityMode {
    FirstComeFirstServe,
    FavourShiestByPointsRaised,
    FavourShiestByTime,
}

pub struct Discussion {
    pub speakers: CaseInsensitiveHashMap<Arc<Mutex<Speaker>>>,
    pub upcoming_speeches: LinkedList<Box<Speech>>,
    pub past_speeches: LinkedList<Box<Speech>>,
    pub duration: Duration,
    pub priority_mode: PriorityMode,
}

impl Discussion {
    
    pub fn new(mover_name: impl Into<UniCase<String>>) -> Self {
        let case_insenstive_mover_name = mover_name.into();
        let mover: Arc<Mutex<Speaker>> = Arc::new(Mutex::new(Speaker::new(case_insenstive_mover_name.clone())));
        let mut speakers: CaseInsensitiveHashMap<Arc<Mutex<Speaker>>> = CaseInsensitiveHashMap::new();
        speakers.insert(case_insenstive_mover_name, Arc::clone(&mover));
        let ret = Self {
            speakers: speakers,
            upcoming_speeches: LinkedList::from([Box::new(Speech::new(mover, false, 0))]),
            past_speeches: LinkedList::from([]),
            duration: Duration::from_secs(0),
            priority_mode: PriorityMode::FirstComeFirstServe,
        };

        return ret;
    }
    
    pub fn add_speech<T: Into<UniCase<String>> + Clone>(&mut self, speaker_name: T, is_response: bool) {

        // TODO this all uses wayyyyyy to much cloning. But also see test2() 
        let speaker: Arc<Mutex<Speaker>> = match self.speakers.get(speaker_name.clone()) {
            Some(speaker_p) => Arc::clone(&speaker_p),
            None => Arc::new(Mutex::new(Speaker::new(speaker_name))),
        };

        // TODO this should not have to be done by the client. It should be automatic
        speaker.lock().unwrap().increment_number_of_speeches_given();

        let new_speech = Box::new(
            Speech::new(
                Arc::clone(&speaker), 
                is_response, 
                self.past_speeches.len() + self.upcoming_speeches.len())
        );
        
        
        match self.priority_mode {
            
            PriorityMode::FirstComeFirstServe => {
                if is_response {
                    // TODO
                    panic!();
                } else {
                    self.upcoming_speeches.push_back(new_speech);
                }
                
            }

            PriorityMode::FavourShiestByPointsRaised => {
                // TODO
                panic!();
            }
            
            PriorityMode::FavourShiestByTime => {
                // TODO
                panic!();
                
                /* 
                let mut checked: LinkedList<Speaker> = LinkedList::new();
                loop {
                    match self.upcoming_speeches.front() {
                        Some(front) => 
                        None => self.upcoming_speeches.push_back(new_speech),
                    }
                }
                */
            }
        }


    }

    pub fn goto_next_speech(&mut self) -> bool {
        match self.upcoming_speeches.pop_front() {
            Some(speech) => {
                self.past_speeches.push_back(speech);
                return true;
            }
            None => return false,
        };
    }

    pub fn goto_previous_speech(&mut self) -> bool {
        match self.past_speeches.pop_back() {
            Some(speech) => {
                self.upcoming_speeches.push_front(speech);
                return true;
            }
            None => return false,
        }; 
    }

    pub fn tick_clock(&mut self) {
        self.duration += Duration::from_secs(1);

        match self.upcoming_speeches.front_mut() {
            Some(speech) => (*speech).tick_clock(),
            None => (),
        };

    }

}

/*
#[test]
fn test1() {
    let disc = Discussion::new("Cici");

    let cici: UniCase<String> = UniCase::from("CICI");
    assert_ne!(disc.speakers.get("Cici"), None);
    assert_ne!(disc.speakers.get("CiCi"), None);
    assert_ne!(disc.speakers.get(cici), None);
    assert_eq!(disc.speakers.get("Grace"), None);

}
*/

#[test]
fn test2() {
    // A little something that's stumping me...
    // Let's make a CaseInsensitiveHashMap with one key-value pair
    let mut m: CaseInsensitiveHashMap<i32> = CaseInsensitiveHashMap::new();
    m.insert("Key", 6);
    let key: UniCase<String> = UniCase::from("key");

    // When I do this, it works:
    m.get(key);

    // However if I replace it with this it does not:
    // m.get(&key);
}