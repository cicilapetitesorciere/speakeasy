use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct NewSpeakerRequest {
    pub name: String,
    pub stype: u8,
}

#[derive(Serialize)]
pub enum Status {
    Normal,
    Paused,
    NonExistant,
    ServerError,
}

#[derive(Serialize)]
pub struct StatusReport {
    pub status: Status,
    pub speaking_order: String,
    pub duration: String,
}

impl StatusReport {
    pub fn default(status: Status) -> Self {
        StatusReport {
            status: status,
            speaking_order: "".to_string(),
            duration: "".to_string(),
        }
    }
}