#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FrameRange {
    pub start_frame: u64,
    pub frame_count: u64,
}

impl FrameRange {
    pub const fn empty() -> Self {
        Self { start_frame: 0, frame_count: 0 }
    }

    pub const fn new(start_frame: u64, frame_count: u64) -> Self {
        Self { start_frame, frame_count }
    }

    pub const fn is_empty(&self) -> bool {
        self.frame_count == 0
    }

    pub const fn end_frame_exclusive(&self) -> u64 {
        self.start_frame.saturating_add(self.frame_count)
    }

    pub const fn contains_frame(&self, frame: u64) -> bool {
        frame >= self.start_frame && frame < self.end_frame_exclusive()
    }

    pub const fn is_adjacent_to(&self, other: &Self) -> bool {
        self.end_frame_exclusive() == other.start_frame || other.end_frame_exclusive() == self.start_frame
    }

    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start_frame < other.end_frame_exclusive() && other.start_frame < self.end_frame_exclusive()
    }

    pub const fn can_coalesce(&self, other: &Self) -> bool {
        self.overlaps(other) || self.is_adjacent_to(other)
    }

    pub const fn coalesce(self, other: Self) -> Self {
        let start = if self.start_frame < other.start_frame { self.start_frame } else { other.start_frame };
        let end = if self.end_frame_exclusive() > other.end_frame_exclusive() { self.end_frame_exclusive() } else { other.end_frame_exclusive() };
        Self { start_frame: start, frame_count: end.saturating_sub(start) }
    }
}

impl Default for FrameRange {
    fn default() -> Self {
        Self::empty()
    }
}
