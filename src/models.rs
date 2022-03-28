#[derive(Clone,Copy)]
pub enum Direction {
    Up,
    Right,
    Left,
    Down,
}

impl Direction {
    // (x,y)
    pub fn to_vector(&self) -> (isize,isize) {
        match self {
            Self::Up => (1,0),
            Self::Right => (0,1),
            Self::Left => (-1,0),
            Self::Down => (0,-1),
        }
    }
}
