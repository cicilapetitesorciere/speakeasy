use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::{HashMap, LinkedList};

use self::linked_list_extra::insert_just_before;
use self::speech::{Speaker, Speech};

mod speech;
mod linked_list_extra;

#[derive(Debug)]
pub enum PriorityMode {
    FirstComeFirstServe,
    FavourBriefest,
    // FavourShiest,
}

#[derive(Debug)]
pub struct Discussion {
    pub speakers: HashMap<String, Arc<Mutex<Speaker>>>,
    pub upcoming_speeches: LinkedList<Box<Speech>>,
    pub past_speeches: LinkedList<Box<Speech>>,
    pub duration: Duration,
    pub paused: bool,
    priority_mode: PriorityMode,
}

impl Discussion {
    
    pub fn new() -> Self {
        let ret = Self {
            speakers: HashMap::new(),
            upcoming_speeches: LinkedList::new(),
            past_speeches: LinkedList::new(),
            duration: Duration::from_secs(0),
            paused: false,
            priority_mode: PriorityMode::FirstComeFirstServe,
        };

        return ret;
    }

    fn insert_block(&self, ll: &mut LinkedList<Box<Speech>>, item: Box<Speech>, block: &mut LinkedList<Box<Speech>>) {
        match self.priority_mode {
                                                
            PriorityMode::FirstComeFirstServe => {
                let threshhold = (*item).fcfs_order;
                block.push_front(item);
                insert_just_before(
                    ll, 
                    block, 
                    |spbx| (*spbx).fcfs_order > threshhold
                );
            },
            
            PriorityMode::FavourBriefest => {
                let threshhold = (*item).speaker.lock().unwrap().total_speaking_time;
                block.push_front(item);
                insert_just_before(
                    ll, 
                    block, 
                    |spbx| (*spbx).speaker.lock().unwrap().total_speaking_time > threshhold
                );
            },

        };
    }

    pub fn resort_speaking_order(&mut self) {

        // First of all we need to store the current speech that is happeneing
        //  seperately from the rest, since it won't be resorted. If there is
        //  no current speech, then clearly the speaking order is vacuously
        //  sorted
        match self.upcoming_speeches.pop_front() {

            Some(current_speech) => {

                // First off, there is a chance that the speaking order begins with some leading responses. Let's begin by handling them
                let mut leading_responses: LinkedList<Box<Speech>> = LinkedList::new();
                loop {
                    
                    match self.upcoming_speeches.pop_front() {
                        
                        Some(response) => if response.is_response {
                            self.insert_block(&mut leading_responses, response, &mut LinkedList::new());
                        } else {
                            self.upcoming_speeches.push_front(response); // Liar! That's no response! Put it back!
                            break;
                        },

                        None => break,
                    
                    }
                
                }

                // Now the first speech should be a new point
                let mut after_leading_responses: LinkedList<Box<Speech>> = LinkedList::new();
                loop {

                    match self.upcoming_speeches.pop_front() {
                        
                        Some(viewed_speech) => {
                            
                            debug_assert!(!viewed_speech.is_response);

                            let mut response_block: LinkedList<Box<Speech>> = LinkedList::new();
                            
                            // Let's first sort the responses 
                            loop {

                                match self.upcoming_speeches.pop_front() {
                                    
                                    Some(response) => {
                                        if (*response).is_response {
                                            self.insert_block(&mut response_block, response, &mut LinkedList::new());
                                        } else {
                                            self.upcoming_speeches.push_front(response);
                                            break;
                                        }
                                    },
                                    
                                    None => break
                                
                                };

                            }
                            
                            self.insert_block(&mut after_leading_responses, viewed_speech, &mut response_block);

                        },

                        None => break

                    };

                }

                self.upcoming_speeches.push_back(current_speech);
                self.upcoming_speeches.append(&mut leading_responses);
                self.upcoming_speeches.append(&mut after_leading_responses);
            }

            None => return (),
        };
    }

    pub fn set_priority_mode(&mut self, mode: PriorityMode) {
        self.priority_mode = mode;
        self.resort_speaking_order();
    }
    
    pub fn add_speech(&mut self, speaker_name: String, is_response: bool) -> bool {

        let speaker: Arc<Mutex<Speaker>> = match self.speakers.get(&speaker_name) {
            Some(speaker_p) => Arc::clone(&speaker_p),
            None => {
                let spkr = Arc::new(Mutex::new(Speaker::new(speaker_name.clone())));
                self.speakers.insert(speaker_name, Arc::clone(&spkr));
                spkr
            },
        };

        let new_speech = Box::new(
            Speech::new(
                Arc::clone(&speaker), 
                is_response, 
                self.past_speeches.len() + self.upcoming_speeches.len())
        );
        
        match self.priority_mode {
            
            // TODO fill out this match statement
            _ => {

                // If we are adding a response, then we need to make sure it gets 
                //  placed just before the next new point, not including the front
                //  of the speaking order
                if is_response {

                    // First we check whether there is in fact anything on the
                    //  speaking order
                    match self.upcoming_speeches.pop_front() {

                        Some(speech_front) => {
                            
                            // If we are currently on a new point, we will 
                            //  temporarily remove it from the list
                            let mut tmp: LinkedList<Box<Speech>> = LinkedList::new();
                            if !speech_front.is_response {
                                tmp.push_back(speech_front);
                            }
                            
                            linked_list_extra::insert_just_before(
                                &mut self.upcoming_speeches, 
                                &mut LinkedList::from([new_speech]), 
                                |spch| !(*spch).is_response);
                            
                            linked_list_extra::prepend(&mut tmp, &mut self.upcoming_speeches);
                            
                            // TODO Remove this once everything else is done properly
                            // self.resort_speaking_order();
                            
                            
                            return true;
                        },

                        // If the speaking order is empty, then there is nothing to respond to, so adding a response doesn't really make a whole lot of sense
                        // TODO: this should be checked by the front end somehow so that it doesn't get to this point
                        // Note that this doesn't actually check that the first speech is a new point, but it shouldn't need to, since how would those responses be added in the first place?
                        None => {
                            // eprintln!("Cannot add response when there is nothing to respond to!");
                            return false;
                        },
                    };
                } else {
                    self.upcoming_speeches.push_back(new_speech);
                    return true;
                }
            }
            
        };

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
        if !self.paused {
            self.duration += Duration::from_secs(1);
            match self.upcoming_speeches.front_mut() {
                Some(speech) => (*speech).tick_clock(),
                None => (),
            };
        }

    }

}

#[test]
fn test1() {
    // TODO fix this test
    let mut discussion = Discussion::new();
    discussion.add_speech("Imane".to_string(), false);
    discussion.add_speech("Cici".to_string(), false);
    discussion.add_speech("Cici".to_string(), true);
    discussion.add_speech("Imane".to_string(), true);

    println!("{:?}", discussion.speakers.keys());
    //assert_eq!(format!("{:?}", discussion.speakers.keys()), "[\"Imane\", \"Cici\"]".to_string());
}