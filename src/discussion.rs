use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::{HashMap, LinkedList};

use debug_panic::debug_panic;

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

#[derive(Debug, Clone, Copy)]
pub enum AddSpeechError {
    ResponseAddedWithNothingToRespondTo,
    ArcLockError,
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

    fn add_existing_speech(&mut self, existing_speech: Box<Speech>) -> Result<(), AddSpeechError> {

        match self.upcoming_speeches.pop_front() {

            Some(speech_front) => {

                let mut result: Result<(), AddSpeechError> = Ok(());

                match self.priority_mode {
                    
                    PriorityMode::FirstComeFirstServe => {

                        if existing_speech.is_response {
                            insert_just_before(
                                &mut self.upcoming_speeches, 
                                &mut LinkedList::from([existing_speech]),
                                |spch| !(*spch).is_response,
                            );
                        } else {
                            self.upcoming_speeches.push_back(existing_speech);
                        }

                    },
                    
                    PriorityMode::FavourBriefest => match Arc::clone(&(*existing_speech).speaker).lock() {

                        Ok(speaker_locked) => {

                            let threshhold = speaker_locked.total_speaking_time;

                            if existing_speech.is_response {
                                insert_just_before(
                                    &mut self.upcoming_speeches,
                                    &mut LinkedList::from([existing_speech]),
                                    |spch| (!spch.is_response || (*&spch).speaker.lock().unwrap().total_speaking_time > threshhold),
                                );
                            } else { 
                                insert_just_before(
                                    &mut self.upcoming_speeches,
                                    &mut LinkedList::from([existing_speech]),
                                    |spch| (!spch.is_response && (*&spch).speaker.lock().unwrap().total_speaking_time > threshhold),
                                );
                            }

                        },

                        Err(_) => result = Err(AddSpeechError::ArcLockError),

                    }

                }

                // It is very important that no return statements appear before
                //  we push speech_front back into the speaking order. If somehow
                //  another return statement ends up in this arm of the match,
                //  that can cause serious bugs. Make sure it gets removed and
                //  that whatever result it was returning gets assigned to the
                //  result variable instead.
                self.upcoming_speeches.push_front(speech_front);
                return result;

            },

            None => if existing_speech.is_response {
                return Err(AddSpeechError::ResponseAddedWithNothingToRespondTo);
            } else {
                self.upcoming_speeches.push_back(existing_speech);
                return Ok(());
            },

        };

    }

    #[allow(unused_variables)]
    pub fn add_new_speech(&mut self, speaker_name: String, is_response: bool) -> Result<(), AddSpeechError> {

        let speaker: Arc<Mutex<Speaker>> = match self.speakers.get(&speaker_name) {
            Some(speaker_p) => Arc::clone(&speaker_p),
            None => {
                let spkr = Arc::new(Mutex::new(Speaker::new(speaker_name.clone())));
                self.speakers.insert(speaker_name, Arc::clone(&spkr));
                spkr
            },
        };

        let new_speech: Box<Speech> = Box::new(
            Speech::new(
                Arc::clone(&speaker), 
                is_response, 
                self.past_speeches.len() + self.upcoming_speeches.len()
            )
        );

        return self.add_existing_speech(new_speech);
        
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
                if let Err(_) = self.add_existing_speech(speech) {
                   debug_panic!();
                }
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
    let _ = discussion.add_new_speech("Imane".to_string(), false);
    let _ = discussion.add_new_speech("Cici".to_string(), false);
    let _ = discussion.add_new_speech("Cici".to_string(), true);
    let _ = discussion.add_new_speech("Imane".to_string(), true);

    println!("{:?}", discussion.speakers.keys());
    //assert_eq!(format!("{:?}", discussion.speakers.keys()), "[\"Imane\", \"Cici\"]".to_string());
}