use crate::entities::profile::Profile;
use std::fs::File;
use std::io::Read;
use crate::io::{get_car_profile_path, get_pedestrian_profile_path, get_bicycle_profile_path};

#[derive(Debug)]
pub enum Error {
    NotAFile,
}

pub fn load_car_profile() -> Result<Profile, Error> {
    return load_profile(get_car_profile_path());
}

pub fn load_pedestrian_profile() -> Result<Profile, Error> {
    return load_profile(get_pedestrian_profile_path());
}

pub fn load_bicycle_profile() -> Result<Profile, Error> {
    return load_profile(get_bicycle_profile_path());
}

fn load_profile(path: &str) -> Result<Profile, Error> {
    let mut file = match File::open(path) {
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