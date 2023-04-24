#[macro_use] extern crate rocket;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use discussion::Discussion;
use lazy_static::lazy_static;
use std::path::Path;
use rocket::fs::NamedFile;
use build_html::*;

use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json;

mod discussion;
mod format_duration;

lazy_static! {
    static ref MDISCUSSIONS: Mutex<HashMap<String, Arc<Mutex<Discussion>>>> = Mutex::new(HashMap::new());
}

fn add_discussion(id: &str) {
    let discp: Arc<Mutex<Discussion>> =  Arc::new(Mutex::new(Discussion::new()));
    
    {  
        let discp_cpy: Arc<Mutex<Discussion>> = Arc::clone(&discp);
        thread::spawn(move || {
            loop {
                let s = Duration::from_secs(1);
                thread::sleep(s);
                (*discp_cpy).lock().unwrap().tick_clock();
            }
        });
    }

    MDISCUSSIONS.lock().unwrap().insert(id.to_string(), discp);
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    if let Ok(file) = NamedFile::open(Path::new("resources/index/index.html")).await {
        return Some(file);
    } else {
        return None;
    }
}

#[get("/resources/<dirname>/<filename>")]
async fn get_resource(dirname: &str, filename: &str) -> Option<NamedFile> {
    if let Ok(file) = NamedFile::open(Path::new(&format!("resources/{}/{}", dirname, filename))).await {
        return Some(file);
    } else {
        return None;
    }
}

#[get("/discussion/<id>")]
async fn get_discussion(id: &str) -> Option<NamedFile> {

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
fn get_speaking_order(id: &str) -> String {
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
fn add_speaker(id: &str, info: &str) {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => {

            let nsr: NewSpeakerRequest = serde_json::from_str(info).unwrap();
            discussion.lock().unwrap().add_speech(nsr.name, nsr.stype == 2);
        }
        None => eprintln!("Error, no discussion named \"{}\".", id),
    };
}

#[post("/discussion/<id>/next")]
fn next(id: &str) {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => discussion.lock().unwrap().goto_next_speech(),
        None => false,
    };
}

#[post("/discussion/<id>/previous")]
fn previous(id: &str) {
    match MDISCUSSIONS.lock().unwrap().get(id) {
        Some(discussion) => discussion.lock().unwrap().goto_previous_speech(),
        None => false,
    };
}

#[get("/favicon.ico")]
fn favicon(){
    return;
}

#[launch]
fn rocket() -> _ {

    rocket::build().mount("/" , routes![
        favicon, 
        index,
        get_resource,
        get_discussion, 
        get_speaking_order,
        add_speaker,
        next,
        previous,
    ])
}