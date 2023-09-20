use uuid::Uuid;

use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn generate_unique_id() -> Result<String, SystemTimeError> {
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH)?.as_secs();
    let random_id = Uuid::new_v4().simple().to_string();
    let uid = format!("{}-{}", timestamp, random_id);
    return Ok(uid);
}

pub enum TimeUnit {
    SECONDS(u64),
    MILLISECONDS(u128),
}

pub struct SECONDS(pub u64);

pub fn get_current_timestamp() -> Result<SECONDS, String> {
    let iat = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Err(e) => return Err(e.to_string()),
        Ok(v) => v,
    };
    let elasped = iat.as_secs();
    return Ok(SECONDS(elasped));
}
