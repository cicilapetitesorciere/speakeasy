#[macro_use] extern crate rocket;

mod discussion;
mod messages;
mod format_duration;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use discussion::{Discussion, PriorityMode};
use discussion::speech::*;
use messages::*;
use lazy_static::lazy_static;
use std::path::Path;
use rocket::fs::NamedFile;
use build_html::*;
use debug_panic::debug_panic;
use format_duration::*;
use serde_json;
use chrono::prelude::*;

lazy_static! {
    static ref MDISCUSSIONS: Mutex<HashMap<String, Arc<Mutex<Discussion>>>> = Mutex::new(HashMap::new());
}

enum GetDiscussionError {
    CouldNotLock,
    NoDiscussionFoundWithGivenID, 
}

fn get_discussion(id: &str) -> Result<Arc<Mutex<Discussion>>, GetDiscussionError> {

    match MDISCUSSIONS.lock() {
        Ok(discussions_hashmap) => match HashMap::get(&*discussions_hashmap, id) {
            Some(discussion) => return Ok(Arc::clone(discussion)),
            None => return Err(GetDiscussionError::NoDiscussionFoundWithGivenID),
        }
        Err(_e) => Err(GetDiscussionError::CouldNotLock),
    }
}

fn add_discussion(id: &str) {

    let new_discussion: Arc<Mutex<Discussion>> =  Discussion::new();

    match MDISCUSSIONS.lock() {

        Ok(mut discussions_hashmap) => {
            HashMap::insert(&mut discussions_hashmap, id.to_string(), new_discussion);
        },

        Err(_) => {
            debug_panic!();
        }

    };

}

#[get("/")]
async fn http_index() -> Option<NamedFile> {
    if let Ok(file) = NamedFile::open(Path::new("resources/index/index.html")).await {
        return Some(file);
    } else {
        return None;
    }
}

#[get("/favicon.ico")]
fn http_favicon() {
    return;
}

#[get("/resources/<dirname>/<filename>")]
async fn http_get_resource(dirname: &str, filename: &str) -> Option<NamedFile> {
    if let Ok(file) = NamedFile::open(Path::new(&format!("resources/{}/{}", dirname, filename))).await {
        return Some(file);
    } else {
        return None;
    }
}

#[get("/discussion/<id>")]
async fn http_get_discussion(id: &str) -> Option<NamedFile> {

    if let Err(GetDiscussionError::NoDiscussionFoundWithGivenID) = get_discussion(id) {
        add_discussion(id);
    }

    if let Ok(file) = NamedFile::open(Path::new("resources/discussion/discussion.html")).await {
        return Some(file);
    } else {
        return None;
    }

}

fn speech_to_html(speech: &Box<Speech>, stype: String) -> [String; 4]{
    match speech.speaker.lock() {
        Ok(speaker) => {
            return [
                speaker.name.clone(),
                stype,
                format_duration_m_s(&speech.duration),
                format_duration_m_s(&speaker.total_speaking_time),
            ];

        }

        Err(e) => {
            debug_panic!(e.to_string());
            [String::new(), String::new(), String::new(), String::new()]
        }

    }
    
}

fn generate_status_report(id: &str) -> Box<StatusReport> {

    return Box::new (
        
        match get_discussion(id) {

            Ok(discussion) => match discussion.lock() {
                
                Ok(locked_discussion) => {

                    let mut speaking_order = Table::new().with_header_row (
                        [
                            "Speaker Name", 
                            "Type", 
                            "Time Speaking", 
                            "Total Speaking Time"
                        ]
                    );

                    if let Some(current_new_point) = &locked_discussion.current_new_point {
                        speaking_order.add_body_row(speech_to_html(current_new_point, "1".to_string()));
                    }

                    for response in &locked_discussion.first_response_block {
                        speaking_order.add_body_row(speech_to_html(&response, "2".to_string()));
                    }
                    
                    for (new_point, responses) in &locked_discussion.upcoming_speeches {
                        speaking_order.add_body_row(speech_to_html(&new_point, "1".to_string()));
                        for response in responses {
                            speaking_order.add_body_row(speech_to_html(&response, "2".to_string()));
                        }
                    }

                    StatusReport {
                        status: if locked_discussion.paused {
                            Status::Paused
                        } else {
                            Status::Normal
                        },
                        speaking_order: speaking_order.to_html_string(),
                        duration: format_duration_m_s(&locked_discussion.duration),
                    }
                
                },

                Err(_) => StatusReport::default(Status::ServerError),

            }

            Err(GetDiscussionError::NoDiscussionFoundWithGivenID) => StatusReport::default(Status::NonExistant),
            
            Err(GetDiscussionError::CouldNotLock) => {
                debug_panic!();
                StatusReport::default(Status::NonExistant)
            },
        }
         
    );   
}

type TimeStamp = i64;

#[get("/discussion/<id>/status")]
fn http_get_status_report(id: &str) -> String {

    unsafe {

        static mut CACHED_REPORT: String = String::new();
        static mut CACHED_AT: TimeStamp = 0;

        let now: TimeStamp = Utc::now().timestamp();
        if now >= CACHED_AT + 1 {  
            CACHED_REPORT = match serde_json::to_string(&generate_status_report(&id)) {
                Ok(json) => json,
                Err(e) => {
                    debug_panic!(e.to_string());
                    "".to_string()
                }
            };
            CACHED_AT = now;
        }

        return CACHED_REPORT.clone();
    
    }

}  


#[post("/discussion/<id>/add_speaker", format="json", data="<info>")]
fn http_add_speaker(id: &str, info: &str) {

    match get_discussion(id) {
        Ok(discussion) => match serde_json::from_str::<NewSpeakerRequest>(info){
            Ok(nsr) => match discussion.lock() {
                Ok(mut locked_discussion) => { 
                    let _ = locked_discussion.add_new_speech(nsr.name, nsr.stype == 2);
                },
                Err(_e) => { debug_panic!(); ()},
            },
            Err(e) => debug_panic!(e),
        },
        Err(e) => debug_panic!(e),
    };
}

#[post("/discussion/<id>/next")]
fn http_next(id: &str) {
    if let Ok(disc) = get_discussion(id) {
        if let Ok(mut disc_locked) = disc.lock() {
            disc_locked.goto_next_speech();
        }
    }

}

#[post("/discussion/<id>/previous")]
fn http_previous(id: &str) {
    if let Ok(disc) = get_discussion(id) {
        if let Ok(mut disc_locked) = disc.lock() {
            disc_locked.goto_previous_speech();
        }
    }
}

#[post("/discussion/<id>/setpause/<state>")]
fn http_pause(id: &str, state: &str) {
    match get_discussion(id) {
        Ok(discussion) => {
            match discussion.lock() {
                Ok(mut locked_discussion) => if state == "pause" {
                    locked_discussion.paused = true;
                } else if state == "unpause" {
                    locked_discussion.paused = false;
                } else {
                    debug_panic!();
                },
                Err(_) => debug_panic!(),
            }
        }
        Err(_) => {
            debug_panic!();
            return ();
        }
    };
}

#[post("/discussion/<id>/set_priority_mode/<mode>")]
fn http_set_priority_mode(id: &str, mode: &str) {
    match get_discussion(id) {
        Ok(discussion) => match discussion.lock() {
            Ok(mut discussion_locked) => discussion_locked.set_priority_mode(match mode {
                "fcfs" => PriorityMode::FirstComeFirstServe,
                "brevity" => PriorityMode::FavourBriefest,
                _ => {
                    debug_panic!();
                    PriorityMode::FirstComeFirstServe
                }
            }),
            Err(_) => debug_panic!(),
        },
        Err(_) => debug_panic!(),
    }
}

#[post("/discussion/<id>/alias/<name1>/<name2>")]
fn http_alias(id: &str, name1: String, name2: String) {
    match get_discussion(id) {
        Ok(discussion) => match discussion.lock() {
            Ok(mut discussion_locked) => discussion_locked.alias_names(&name1, &name2),
            Err(_) => debug_panic!(),
        }
        Err(_) => debug_panic!(),
    }
}

#[launch]
fn rocket() -> _ {

    rocket::build().mount("/" , routes![
        http_favicon, 
        http_index,
        http_get_resource,
        http_get_discussion, 
        http_get_status_report,
        http_add_speaker,
        http_next,
        http_previous,
        http_pause,
        http_set_priority_mode,
        http_alias,
    ])

}