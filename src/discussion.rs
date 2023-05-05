mod linked_list_extra;

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

#[derive(Debug, PartialEq, Eq)]
pub enum GotoSpeechResult {
    Success,
    NoSpeechToGoTo,
    IllegalDiscussionSomehow,
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

    /*
    pub fn alias_names(&mut self, name1: &String, name2: &String) {

        // First we try and find speakers with name1 and name2. If either of them 
        //  don't exist, then we create it
        let [speaker1_arc, speaker2_arc]: [Arc<Mutex<Speaker>>; 2] = [name1, name2].map(|name: &String| self.speakers.get(name).map(Arc::clone).unwrap_or_else(||Arc::new(Mutex::new(Speaker::new((&name).to_string())))));

        // We now need to make sure these two speakers we found are distict. If
        //  they aren't, then we can end the function now because the two speakers
        //  are already trivially aliased
        if !Arc::ptr_eq(&speaker1_arc, &speaker2_arc) {

            // We then insert a reference to speaker1 anywhere that speaker2 was 
            //  (or should have been)
            match speaker2_arc.lock() {
                Ok(speaker2) => {
                    self.speakers.insert(String::clone(&speaker2.name), Arc::clone(&speaker1_arc));
                    for name in &speaker2.aliases {
                        self.speakers.insert(String::clone(&name), Arc::clone(&speaker1_arc));
                    }
                },
                Err(e) => debug_panic!(e.to_string()),
            };
    
            // This next block will now go through each speech in the past,
            //  present, and future speaking order, replacing any instance of
            //  speaker2 with one to speaker1
            {
    
                let speaker1_to_speaker2 = |speech: &mut Speech| if Arc::ptr_eq(&speech.speaker, &speaker2_arc) {
                    speech.speaker = Arc::clone(&speaker1_arc);
                };
    
                if let Some(speech) = &mut self.current_new_point {
                    speaker1_to_speaker2(speech);
                }
    
                for speech in &mut self.first_response_block {
                    speaker1_to_speaker2(speech);
                }
                
                for (new_point, response_block) in Iterator::chain(self.past_speeches.iter_mut(), self.upcoming_speeches.iter_mut()) {
                    speaker1_to_speaker2(new_point);
                    for response in response_block {
                        speaker1_to_speaker2(response);
                    }
                }
    
            }
    
            // Finally, we should have scrubbed all references to speaker2 from
            //  the discussion, leaving only the one reference scoped to this 
            //  function
            if let (Ok(mut speaker1), Ok(Ok(speaker2))) = (speaker1_arc.lock(), Arc::try_unwrap(speaker2_arc).map(Mutex::into_inner)) {
                speaker1.merge_with(speaker2);
            } else {
                debug_panic!();
            }

        }

    }
    */

    pub fn add_new_speech(&mut self, speaker_name: String, is_response: bool) {

        // We are given a name and we need to turn that into a speaker object. We
        //  first check the list of speakers to see if a speaker with that name
        //  already exists. If it does we create a new pointer to them. Otherwise
        //  we create a new speaker
        let speaker: Arc<Mutex<Speaker>> = match self.speakers.get(&speaker_name) {
            Some(speaker_p) => Arc::clone(&speaker_p),
            None => {
                let spkr = Arc::new(Mutex::new(Speaker::new(speaker_name.clone())));
                self.speakers.insert(speaker_name, Arc::clone(&spkr));
                spkr
            },
        };

        // We then create a new speech with the speaker from the previous step
        let new_speech: Box<Speech> = Box::new(
            Speech{
                speaker: Arc::clone(&speaker), 
                duration: ZERO_SECONDS, 
                fcfs_order: self.past_speeches.len() + self.upcoming_speeches.len()
            }
        );

        // We then add it to the speaking order in a way that makes sense
        if is_response {
            if self.current_new_point.is_some() || !self.first_response_block.is_empty() {
                self.first_response_block.push_back(new_speech);
            }
        } else if self.current_new_point.is_none() && self.first_response_block.is_empty() {
            self.current_new_point = Some(new_speech);
        } else {
            self.upcoming_speeches.push_back((new_speech, LinkedList::new()));
        }

        // Finally, we may need to resort the speaking order so that our new
        //  speech ends up in the correct position.
        if self.priority_mode != PriorityMode::FirstComeFirstServe {
            self.resort_speaking_order();
        }
        
    }

    pub fn goto_next_speech(&mut self) -> GotoSpeechResult {
        
        //////////////////////////////////////////////////////////////////////////
        //
        // There are a few cases we need to consider:
        //
        //   1. Moving from a new point to one of its responses
        //      - This will occur if `current_new_point` is `Some(_)` and
        //         `first_response_block` is non-empty. 
        //      - In this case `current_new_point` becomes `None`, and the 
        //         previous value of `current_new_point` needs to be moved to 
        //        `past_speeches` with an empty list as its responses
        //
        //   2. Moving from one response to the next response
        //      - This will occur if `current_new_point` is `None` and there are
        //         at least two elements in `first_response_block` (the current
        //         response and the one we will be moving to). 
        //      - We will need to move the first response to the back of the most
        //         recent response block
        //      - There is a weird contigency here in that it may be the case that
        //         there is no past speeches. This should never happen unless
        //         there is a bug, but we should catch it regardless
        //
        //   3. Moving from the last in a chain of responses to another new point
        //      - This will occur if `current_new_point` is `None`, and there is
        //         only one response in `first_response_block`.
        //      - We will need to move the current response to the back of the
        //         most recent response block, leaving it empty. Then we will
        //         replace `current_new_point` and `first_response_block` with
        //         items from the head of `upcoming_speeches`.
        //      - This deals with a similar contigency as the previous case
        //      - There is also the contingency that there are no upcoming
        //         speeches. In this case we can just keep the current new point
        //         and first response block as empty
        //
        //   4. Moving from a new point with no responses to the next new point
        //      - This will occur if `current_new_point` is `Some(_)` and 
        //         `first_response_block` empty.
        //      - We will have to replace the value of `(current_new_point,
        //         first_response_block)` with the head of `upcoming_speeches`,
        //         and move the old value to the back of `past_speeches`
        // 
        //  As well as the following edge case:
        //      
        //      5. It may be the case that `current_new_point` is `None`, and that
        //          there are no responses in `first_response_block`. This is in
        //          fact the initial state of any discussion. If this is the case,
        //          then it should also be true that `upcoming_speeches` is empty,
        //          otherwise we've hit some kind of bug. Assuming that
        //          `upcoming_speeches` is in fact empty, we do nothing.
        //
        //////////////////////////////////////////////////////////////////////////

        match mem::take(&mut self.current_new_point) {
            Some(old_cnp) => if self.first_response_block.is_empty() {
                // Case 4
                match self.upcoming_speeches.pop_front() {
                    Some((next_np, next_rb)) => {
                        self.current_new_point = Some(next_np);
                        self.first_response_block = next_rb;
                        self.past_speeches.push_back((old_cnp, LinkedList::new()));
                        return GotoSpeechResult::Success;
                    },
                    None => {
                        self.current_new_point = None;
                        self.first_response_block = LinkedList::new();
                        self.past_speeches.push_back((old_cnp, LinkedList::new()));
                        return GotoSpeechResult::NoSpeechToGoTo;
                    },
                }
            } else {
                // Case 1
                self.current_new_point = None;
                self.past_speeches.push_back((old_cnp, LinkedList::new()));
                return GotoSpeechResult::Success;
            },

            None => match self.first_response_block.pop_front() {
                Some(current_response) => match self.past_speeches.back_mut() {
                    Some((_, previous_response_block)) => {
                        
                        // Case 2 and 3 first step
                        previous_response_block.push_back(current_response);


                        if self.first_response_block.is_empty() {
                            // Case 3
                            match self.upcoming_speeches.pop_front() {
                                Some((next_np, next_rb)) => {
                                    self.current_new_point = Some(next_np);
                                    self.first_response_block = next_rb;
                                    return GotoSpeechResult::Success;
                                }
                                None => return GotoSpeechResult::NoSpeechToGoTo,
                            }
                        }
                    },
                    None => {
                        debug_panic!();
                        self.past_speeches.push_back((current_response, LinkedList::new()));
                        return GotoSpeechResult::IllegalDiscussionSomehow;
                    },
                }
                None => match self.upcoming_speeches.pop_front() {
                    Some((uh_wtf, why)) => {
                        debug_panic!();
                        self.current_new_point = Some(uh_wtf);
                        self.first_response_block = why;
                        return GotoSpeechResult::Success;
                    },
                    None => return GotoSpeechResult::NoSpeechToGoTo,
                }
            }
        }
        return GotoSpeechResult::IllegalDiscussionSomehow;
    }

    // TODO make this return a GotoSpeechResult and utilize it in the frontend
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
                None => if let Some(current_new_point) = mem::replace(&mut self.current_new_point, Some(most_recent_new_point)) {
                    debug_assert!(most_recent_response_block.is_empty());
                    self.upcoming_speeches.push_front((current_new_point, mem::replace(&mut self.first_response_block, most_recent_response_block)));
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