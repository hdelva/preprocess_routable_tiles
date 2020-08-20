use std::str::FromStr;

pub enum Areas {
    Belgium,
    London,
    Pyrenees,
    Dummy,
}

impl FromStr for Areas {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "belgium" => Ok(Areas::Belgium),
            "london" => Ok(Areas::London),
            "pyrenees" => Ok(Areas::Pyrenees),
            "dummy" => Ok(Areas::Dummy),
            _ => Err("no match"),
        }
    }
}
