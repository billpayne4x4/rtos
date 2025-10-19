pub mod display;
pub mod scale;

pub use scale::scale_raw_nearest_into;

#[cfg(feature = "alloc")]
pub use scale::scale_raw_nearest;
