#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileCoordinate {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

impl TileCoordinate {
    pub fn new(x: u32, y: u32, zoom: u32) -> TileCoordinate {
        TileCoordinate {x, y, zoom}
    }

    pub fn get_children(&self) -> [TileCoordinate; 4] {
        [
            TileCoordinate::new(self.x * 2,     self.y * 2,     self.zoom + 1),
            TileCoordinate::new(self.x * 2 + 1, self.y * 2,     self.zoom + 1),
            TileCoordinate::new(self.x * 2,     self.y * 2 + 1, self.zoom + 1),
            TileCoordinate::new(self.x * 2 + 1, self.y * 2 + 1, self.zoom + 1),
        ]
    }
}