#[derive(Debug, Copy, Clone)]
pub struct Interval(pub f32, pub f32);

impl Interval {
    pub fn contains(self, t: f32) -> bool {
        self.0 <= t && t <= self.1
    }

    pub fn overlaps(self, other: Interval) -> bool {
        self.0 <= other.1 && other.0 <= self.1
    }
}
