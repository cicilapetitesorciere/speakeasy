use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::{HashMap, LinkedList};
use std::mem;

use debug_panic::debug_panic;

use self::speech::{Speaker, Speech};

pub mod speech;

const ZERO_SECONDS: Duration = Duration::from_secs(0);
const ONE_SECOND: Duration = Duration::from_secs(1);

#[derive(Debug, PartialEq, Eq)]
pub enum PriorityMode {
    FirstComeFirstServe,
    FavourBriefest,
    // FavourShiest,
}

type ResponseBlock = LinkedList<Box<Speech>>;
type NewPointThenResponseBlock = (Box<Speech>, ResponseBlock);
type ListOfSpeeches = LinkedList<NewPointThenResponseBlock>;

#[derive(Debug)]
pub struct Discussion {
    pub speakers: HashMap<String, Arc<Mutex<Speaker>>>, 
    pub current_new_point: Option<Box<Speech>>,
    pub first_response_block: ResponseBlock, // An empty response block represents a paused discussion
    pub upcoming_speeches: ListOfSpeeches,
    pub past_speeches: ListOfSpeeches,
    pub duration: Duration,
    pub paused: bool,
    pub as_html: String,
    priority_mode: PriorityMode,
}

impl Discussion {
    
    pub fn new() -> Arc<Mutex<Self>> {

        let ret: Arc<Mutex<Self>> = Arc::new (
            Mutex::new (
                Self {
                    speakers: HashMap::new(),
                    current_new_point: None,
                    first_response_block: LinkedList::new(),
                    upcoming_speeches: LinkedList::new(),
                    past_speeches: LinkedList::new(),
                    duration: ZERO_SECONDS,
                    paused: false,
                    as_html: "".to_string(),
                    priority_mode: PriorityMode::FirstComeFirstServe,
                }
            )
        );

        let ret_clock_pointer: Arc<Mutex<Self>> = Arc::clone(&ret);

        thread::spawn(move || {

            while Arc::strong_count(&ret_clock_pointer) > 1 {
            
                thread::sleep(ONE_SECOND);
                
                match (*ret_clock_pointer).lock() {
                    Ok(mut ret_locked) => {
                        ret_locked.tick_clock();
                        if ret_locked.priority_mode != PriorityMode::FirstComeFirstServe {
                            ret_locked.resort_speaking_order();
                        }
                    },
                    Err(e) => debug_panic!(e.to_string()),
                }

            }

        });

        return ret;

    }

    fn sort_response_block(block: &mut ResponseBlock, is_more_pressing: fn(&Box<Speech>, &Box<Speech>) -> bool) {

        // We will create a temporary variable to store the items from `block` after
        //  we've looked at each of them. This will eventually become `block` itself
        let mut tmp1: ResponseBlock = LinkedList::new();
        loop {
            match block.pop_front() {
                Some(response) => {
                    let mut tmp2: ResponseBlock = LinkedList::new();
                    loop {
                        match tmp1.pop_back() {
                            Some(tmp1_back) => {
                                if is_more_pressing(&response, &tmp1_back) {
                                    tmp2.push_front(tmp1_back);
                                } else {
                                    tmp1.push_back(tmp1_back);
                                    break;
                                }
                            },
                            None => break,
                        }
                    }
                    tmp1.push_back(response);
                    tmp1.append(&mut tmp2);
                    debug_assert!(tmp2.is_empty());
                    drop(tmp2);
                }
                None => {
                    block.append(&mut tmp1);
                    debug_assert!(tmp1.is_empty());
                    return;
                },
            }
        }
    }
    
    pub fn resort_speaking_order(&mut self) {

        // First we define a function which compares two speeches `a` and `b` and
        //  returns true if `a` should be placed before `b`.
        let is_more_pressing: fn(&Box<Speech>, &Box<Speech>) -> bool = match self.priority_mode {

            PriorityMode::FirstComeFirstServe => |a: &Box<Speech>, b: &Box<Speech>| -> bool {
                return a.fcfs_order < b.fcfs_order;
            },

            PriorityMode::FavourBriefest => |a: &Box<Speech>, b: &Box<Speech>| -> bool {

                let a_total_speaking_time: Duration = match a.speaker.lock() {
                    Ok(speaker) => speaker.total_speaking_time,
                    Err(e) => {
                        debug_panic!(e.to_string());
                        return false;
                    }
                };

                let b_total_speaking_time: Duration = match b.speaker.lock() {
                    Ok(speaker) => speaker.total_speaking_time,
                    Err(e) => {
                        debug_panic!(e.to_string());
                        return false;
                    }
                };

                return a_total_speaking_time < b_total_speaking_time;

            },

        };

        // Now we move onto the meat of the function
        
        // We will go one by one and move the entire list of upcoming speeches
        //  into `tmp1`, sorting them as we go. Note that tmp1 wi
        let mut tmp1: ListOfSpeeches = LinkedList::new();
        loop {
            match self.upcoming_speeches.pop_front() {
                Some((new_point, mut response_block)) => {

                    // TODO make the sorting of the block and the finding location
                    //  to inser the pair into two seperate threads

                    // Thread 1
                    {
                        Self::sort_response_block(&mut response_block, is_more_pressing);
                    }

                    // Thread 2
                    let mut tmp2: ListOfSpeeches = LinkedList::new();
                    loop {

                        match tmp1.pop_back() {

                            Some(tmp1_back) => {
                                if is_more_pressing(&new_point, &tmp1_back.0) {
                                    tmp2.push_front(tmp1_back);
                                } else {
                                    tmp1.push_back(tmp1_back);
                                    break;
                                }
                            },

                            None => break,
                        }

                    }

                    tmp1.push_back((new_point, response_block));
                    tmp1.append(&mut tmp2);
                    debug_assert!(tmp2.is_empty());
                    drop(tmp2);

                },

                None => {
                    self.upcoming_speeches.append(&mut tmp1);
                    debug_assert!(tmp1.is_empty());
                    return;
                },

            }
        }

    }

    pub fn set_priority_mode(&mut self, mode: PriorityMode) {
        self.priority_mode = mode;
        self.resort_speaking_order();
    }

    pub fn add_new_speech(&mut self, speaker_name: String, is_response: bool) {

        let speaker: Arc<Mutex<Speaker>> = match self.speakers.get(&speaker_name) {
            Some(speaker_p) => Arc::clone(&speaker_p),
            None => {
                let spkr = Arc::new(Mutex::new(Speaker::new(speaker_name.clone())));
                self.speakers.insert(speaker_name, Arc::clone(&spkr));
                spkr
            },
        };

        let new_speech: Box<Speech> = Box::new(
            Speech{
                speaker: Arc::clone(&speaker), 
                duration: ZERO_SECONDS, 
                fcfs_order: self.past_speeches.len() + self.upcoming_speeches.len()
            }
        );

        if is_response {
            self.first_response_block.push_back(new_speech);
        } else if self.current_new_point.is_none() && self.first_response_block.is_empty() {
            self.current_new_point = Some(new_speech);
        } else {
            self.upcoming_speeches.push_back((new_speech, LinkedList::new()));
        }

        if self.priority_mode != PriorityMode::FirstComeFirstServe {
            self.resort_speaking_order();
        }
        
    }

    pub fn goto_next_speech(&mut self) {

        match mem::replace(&mut self.current_new_point, None) {

            Some(old_current_new_point) => self.past_speeches.push_back((old_current_new_point, LinkedList::new())),

            None => match (self.first_response_block.pop_front(), self.past_speeches.back_mut()) {
                (Some(current_response), Some((_, most_recent_response_block))) => {
                    most_recent_response_block.push_back(current_response);
                },
                (None, _) => (),
                (Some(current_response), None) => {
                    eprintln!("You are trying to move a current response into past speeches, but there are no past speeches. Which doens't make a lot of sense. What was the current speech a response to.");
                    debug_panic!();
                    eprintln!("If we were in debug mode I would have panicked jut now, but clearly we're live, so I'm going to try and solve this by making the current speech into a new point. Voila. You officially have a bug!");
                    self.past_speeches.push_back((current_response, LinkedList::new()));
                },
            },

        }

        if self.current_new_point.is_none() && self.first_response_block.is_empty() {
            if let Some((next_new_point, next_response_block)) = self.upcoming_speeches.pop_front() {
                self.current_new_point = Some(next_new_point);
                self.first_response_block = next_response_block;
            }
        }

    }

    pub fn goto_previous_speech(&mut self) {
        if let Some((most_recent_new_point, mut most_recent_response_block)) = self.past_speeches.pop_back() {
            match most_recent_response_block.pop_back() {
                Some(most_recent_response) => {
                    // If the current speech is a new point, then we need to push both it and all its responses into the upcoming speeches
                    if let Some(current_new_point) = mem::replace(&mut self.current_new_point, None) {
                        self.upcoming_speeches.push_front((current_new_point, mem::replace(&mut self.first_response_block, LinkedList::new())));
                    }
                    self.first_response_block.push_front(most_recent_response);
                    self.past_speeches.push_back((most_recent_new_point, most_recent_response_block));
                },
                None => {
                    if let Some(current_new_point) = mem::replace(&mut self.current_new_point, Some(most_recent_new_point)) {
                        debug_assert!(most_recent_response_block.is_empty());
                        self.upcoming_speeches.push_front((current_new_point, mem::replace(&mut self.first_response_block, most_recent_response_block)));
                    }
                },
            }
        }
    }

    pub fn tick_clock(&mut self) {
        if !self.paused {
            self.duration += ONE_SECOND;
            match &mut self.current_new_point {
                Some(current_new_point) => current_new_point.tick_clock(),
                None => if let Some(current_response) = &mut self.first_response_block.front_mut() {
                    current_response.tick_clock();
                },
            }
        }
    }
}