#[derive(Clone, Copy)]
pub enum RtkError {
    BlobTooSmall,
    BadMagic,
    BadHeaderLen,
    SegmentTableOutOfBounds,
}
