use crate::entities::profile::Profile;
use std::fs::File;
use std::io::Read;
use crate::io::get_profile_path;

#[derive(Debug)]
pub enum Error {
    NotAFile,
}

pub fn load_profile() -> Result<Profile, Error> {
    let mut file = match File::open(get_profile_path()) {
        Ok(file) => file,
        _ => return Err(Error::NotAFile),
    };

    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Err(_) => return Err(Error::NotAFile),
        _ => {},
    };

    Ok(serde_json::from_str(&data).unwrap())
}