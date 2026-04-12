#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Score(f32);

impl Eq for Score {}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("Could not compare scores")
    }
}

impl From<f32> for Score {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
