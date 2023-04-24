
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::{HashMap, LinkedList};

use self::speech::{Speaker, Speech};

mod speech;

#[derive(Debug)]
pub enum PriorityMode {
    FirstComeFirstServe,
    FavourShiestByPointsRaised,
    FavourShiestByTime,
}

#[derive(Debug)]
pub struct Discussion {
    pub speakers: HashMap<String, Arc<Mutex<Speaker>>>,
    pub upcoming_speeches: LinkedList<Box<Speech>>,
    pub past_speeches: LinkedList<Box<Speech>>,
    pub duration: Duration,
    pub paused: bool,
    pub priority_mode: PriorityMode,
}


impl Discussion {
    
    pub fn new() -> Self {
        let ret = Self {
            speakers: HashMap::new(),
            upcoming_speeches: LinkedList::from([]),
            past_speeches: LinkedList::from([]),
            duration: Duration::from_secs(0),
            paused: false,
            priority_mode: PriorityMode::FirstComeFirstServe,
        };

        return ret;
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

        // There is a decent chance we will be doing some traversal over the
        //  speaking order to find a suitible place to put our new speech. Let's
        //  make a little spot to put things after we've checked them.
        let mut checked: LinkedList<Box<Speech>> = LinkedList::new();
        
        // There are a lot of things that may happen in this match block, but
        //  ultimately one of three things will happen:
        //
        // 1. We will have found that adding the new speech is not possible, and
        //     will have returned a false
        //
        // 2. We will have added the new speech into the speaking order in its
        //      rightful place and returned true
        //
        // 3. The variable "checked" will contain all the speeches that belong
        //      before the one we are inserting, and self.upcoming_speeches will
        //      contain everything that belongs after 
        //
        match self.priority_mode {
            
            PriorityMode::FirstComeFirstServe => {

                // If we are adding a response, then we need to make sure it gets 
                //  placed just before the next new point, not including the front
                //  of the speaking order
                if is_response {

                    // First we check whether there is in fact anything on the
                    //  speaking order
                    match self.upcoming_speeches.pop_front() {

                        Some(speech_front) => {
                            
                            // If we are currently on a new point, we will add 
                            //  that to the checked list, since it will not be 
                            //  caught by the coming loop
                            if (!speech_front.is_response){
                                checked.push_back(speech_front);
                            }
                            

                            // Now we keep trying to pop the front off the list. 
                            //  There are two termination conditions here:
                            //
                            //      1. We reach the end of the list. In other words, 
                            //          there are not yet any new points after the 
                            //          current one, and hence the response just
                            //          gets added to the end of the speaking 
                            //          order
                            //
                            //      2. We reach a new point, in which case the 
                            //          response will be added just before it 
                            loop {

                                match self.upcoming_speeches.pop_front() {

                                    Some(viewed_speech) => {
                                        if (*viewed_speech).is_response {
                                            checked.push_back(viewed_speech);
                                        } else {
                                            self.upcoming_speeches.push_front(viewed_speech);
                                            break;
                                        }
                                    },
                                    
                                    None => {
                                        break;
                                    },
                                };
                            }
                        },

                        // If the speaking order is empty, then there is nothing to respond to, so adding a response doesn't really make a whole lot of sense
                        // TODO: this should be checked by the front end somehow so that it doesn't get to this point
                        // Note that this doesn't actually check that the first speech is a new point, but it shouldn't need to, since how would those responses be added in the first place?
                        None => {
                            eprintln!("Cannot add response when there is nothing to respond to!");
                            return false;
                        },
                    };
                } else {
                    self.upcoming_speeches.push_back(new_speech);
                    return true;
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
        };

        // If we are reaching this part of the function, then good news: the speech
        //  can be added. Plus we also have the place it belongs exposed for us.
        //
        //  Hooray! 
        //
        //  Now all there is left to do is sandwiching our new speach between
        //    checked and self.upcoming_speeches
        checked.push_back(new_speech);
        checked.append(&mut self.upcoming_speeches);
        self.upcoming_speeches = checked;
        return true;


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
    let mut discussion = Discussion::new();
    discussion.add_speech("Imane".to_string(), false);
    discussion.add_speech("Cici".to_string(), false);
    discussion.add_speech("Cici".to_string(), true);
    discussion.add_speech("Imane".to_string(), true);

    assert_eq!(format!("{:?}", discussion.speakers.keys()), "[\"Cici\", \"Imane\"]".to_string());
}