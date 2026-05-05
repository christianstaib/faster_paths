use std::ops::Add;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct VertexId(u32);

impl VertexId {
    pub fn new(vertex_id: u32) -> Self {
        Self(vertex_id)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Distance(u32);

impl Add for Distance {
    type Output = Distance;

    fn add(self, rhs: Distance) -> Self::Output {
        Distance(self.0 + rhs.0)
    }
}

impl Distance {
    pub const ZERO: Self = Self(0);

    pub fn new(distance: u32) -> Self {
        Self(distance)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}
