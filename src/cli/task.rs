use std::{fmt, str::FromStr};

pub enum Task {
    ReduceProfile,
    ReduceTransit,
    ReducePaddedTransit,
    ReduceBinary,
    Merge,
    FetchTiles
}

impl fmt::Display for Task {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Task::ReduceProfile => write!(f, "reduce_profile"),
            _ => write!(f, "reduce_profile")
        }
    }
}

impl FromStr for Task {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reduce_profile" => Ok(Task::ReduceProfile),
            "reduce_transit" => Ok(Task::ReduceTransit),
            "reduce_padded_transit" => Ok(Task::ReducePaddedTransit),
            "reduce_binary" => Ok(Task::ReduceBinary),
            "merge" => Ok(Task::Merge),
            "fetch_tiles" => Ok(Task::FetchTiles),
            _ => Err("no match"),
        }
    }
}
