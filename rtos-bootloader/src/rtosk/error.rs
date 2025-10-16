#[derive(Clone, Copy)]
pub enum RtoskError {
    BlobTooSmall,
    BadMagic,
    BadHeaderLen,
    SegmentTableOutOfBounds,
}
