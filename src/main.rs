#[macro_use] extern crate rocket;

use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use discussion::Discussion;
use lazy_static::lazy_static;
use std::path::Path;
use rocket::fs::NamedFile;
use build_html::*;
use debug_panic::debug_panic;

use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json;

mod discussion;
mod ongoing_discussions_list;
mod format_duration;

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
    let discp: Arc<Mutex<Discussion>> =  Arc::new(Mutex::new(Discussion::new()));
    let discp_cpy: Arc<Mutex<Discussion>> = Arc::clone(&discp);
    
    MDISCUSSIONS.lock().unwrap().insert(id.to_string(), discp);
    
    thread::spawn(move || {
        loop {
            let s = Duration::from_secs(1);
            thread::sleep(s);
            (*discp_cpy).lock().unwrap().tick_clock();
        }
    });

}

#[get("/")]
async fn http_index() -> Option<NamedFile> {
    if let Ok(file) = NamedFile::open(Path::new("resources/index/index.html")).await {
        return Some(file);
    } else {
        return None;
    }
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

    if !MDISCUSSIONS.lock().unwrap().contains_key(id) {
        add_discussion(id);
    }

    if let Ok(file) = NamedFile::open(Path::new("resources/discussion/discussion.html")).await {
        return Some(file);
    } else {
        return None;
    }
}

#[get("/discussion/<id>/speaking_order.html")]
fn http_get_speaking_order(id: &str) -> String {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => {
            let mut ret = Table::new().with_header_row(["Speaker Name", "Type", "Time Speaking", "Total Speaking Time"]);
            for speech in &discussion.lock().unwrap().upcoming_speeches {
                let speaker = speech.speaker.lock().unwrap();
                ret.add_body_row([
                    speaker.name.to_string(),
                    (if speech.is_response {2} else {1}).to_string(),
                    format_duration::format_duration_M_S(&speech.duration),
                    format_duration::format_duration_M_S(&speaker.total_speaking_time),
                ]);
            }
            return ret.to_html_string();
        },
        None => return "".to_string(),
    };
}

#[derive(Deserialize)]
struct NewSpeakerRequest {
    name: String,
    stype: u8,
}

#[post("/discussion/<id>/add_speaker", format="json", data="<info>")]
fn http_add_speaker(id: &str, info: &str) {

    match get_discussion(id) {
        Ok(discussion) => match serde_json::from_str::<NewSpeakerRequest>(info){
            Ok(nsr) => match discussion.lock() {
                Ok(mut locked_discussion) => { locked_discussion.add_speech(nsr.name, nsr.stype == 2); ()},
                Err(_e) => { debug_panic!(); ()},
            },
            Err(e) => debug_panic!(e),
        },
        Err(e) => debug_panic!(e),
    };
}

#[post("/discussion/<id>/next")]
fn http_next(id: &str) {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => discussion.lock().unwrap().goto_next_speech(),
        None => false,
    };
}

#[post("/discussion/<id>/previous")]
fn http_previous(id: &str) {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => discussion.lock().unwrap().goto_previous_speech(),
        None => false,
    };
}

#[get("/favicon.ico")]
fn http_favicon(){
    return;
}

#[launch]
fn rocket() -> _ {

    rocket::build().mount("/" , routes![
        http_favicon, 
        http_index,
        http_get_resource,
        http_get_discussion, 
        http_get_speaking_order,
        http_add_speaker,
        http_next,
        http_previous,
    ])
}