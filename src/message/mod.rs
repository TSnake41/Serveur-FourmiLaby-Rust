/// # Transmission primitives
/// 
/// Transmission currently works using a u32 big-endian number that corresponds to
/// the size of the payload message, followed by this payload message that is a JSON
/// [`Message`] object.
///
pub mod transmit;

/// Message structures.
pub mod types;
