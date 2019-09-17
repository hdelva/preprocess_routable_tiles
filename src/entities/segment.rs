pub struct Segment<'a> {
    pub from: &'a str,
    pub to: &'a str,
}

pub struct WeightedSegment<'a> {
    pub segment: Segment<'a>,
    pub weight: u64,
}

impl<'a> Segment<'a> {
    pub fn new(from: &'a str, to: &'a str) -> Segment<'a> {
        Segment { from, to }
    }
}

impl<'a> WeightedSegment<'a> {
    pub fn new(segment: Segment<'a>, weight: u64) -> WeightedSegment<'a> {
        WeightedSegment { segment, weight }
    }
}