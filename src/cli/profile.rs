use std::str::FromStr;

pub enum Profiles {
    Car,
    Bicycle,
    Pedestrian,
}

impl FromStr for Profiles {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "car" => Ok(Profiles::Car),
            "bicycle" => Ok(Profiles::Bicycle),
            "pedestrian" => Ok(Profiles::Pedestrian),
            _ => Err("no match"),
        }
    }
}
