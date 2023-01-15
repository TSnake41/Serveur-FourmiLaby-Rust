//! # Transmission primitives
//!
//! Transmission currently works using a [`u32`] big-endian number that corresponds to
//! the size of the payload message, followed by this payload message that is a JSON
//! [`super::types::Message`] object.
//! 
use async_std::io::{Read, Write, WriteExt, ReadExt};

use super::types::Message;
use crate::error::ServerError;

const MAX_MESSAGE_SIZE: u32 = 4 * 1024 * 1024; // 4 MiB

/// Write a [Message] to `writer` using the protocol.
pub async fn write_message<W: Write + Unpin>(writer: &mut W, message: &Message) -> Result<(), ServerError> {
    write_message_raw(writer, serde_json::to_string(message)?.as_bytes()).await
}

/// Write raw data to `writer` using the protocol.
pub async fn write_message_raw<T: Write + Unpin>(writer: &mut T, data: &[u8]) -> Result<(), ServerError> {
    let data_len = u32::try_from(data.len()).or_else(|_| {
        ServerError::transmission_error(format!(
            "Data is too large ! ({} > {})",
            data.len(),
            u32::MAX
        ))
    })?;

    writer.write_all(&data_len.to_be_bytes()).await?;
    writer.write_all(data).await?;

    Ok(())
}

/// Read raw data from `reader` using the protocol.
pub async fn read_message_raw<R: Read + Unpin>(reader: &mut R) -> Result<Box<[u8]>, ServerError> {
    let mut data_len_buffer = [0u8; 4];

    reader.read_exact(&mut data_len_buffer).await?;

    let data_len = u32::from_be_bytes(data_len_buffer);

    if data_len > MAX_MESSAGE_SIZE {
        return ServerError::transmission_error(format!(
            "Received message is too big ! ({data_len} > {MAX_MESSAGE_SIZE})"
        ));
    }

    let mut data = vec![0u8; data_len as usize];

    reader.read_exact(data.as_mut_slice()).await?;

    Ok(data.into_boxed_slice())
}

//TODO: Use Read::take() and BufReader instead ?

/// Read a [Message] from `reader` using the protocol.
pub async fn read_message<R: Read + Unpin>(reader: &mut R) -> Result<Message, ServerError> {
    let data = read_message_raw(reader).await?;

    let message = serde_json::from_slice::<Message>(&data)?;

    Ok(message)
}
