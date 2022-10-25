use std::io::{Read, Write};

use super::types::Message;
use crate::error::ServerError;

const MAX_MESSAGE_SIZE: u32 = 4 * 1024 * 1024; // 4 MiB

pub fn write_message<W: Write>(writer: &mut W, message: &Message) -> Result<(), ServerError> {
    write_message_raw(writer, serde_json::to_string(message)?.as_bytes())
}

pub fn write_message_raw<T: Write>(writer: &mut T, data: &[u8]) -> Result<(), ServerError> {
    let data_len = u32::try_from(data.len()).or_else(|_| {
        ServerError::transmission_error(format!(
            "Data is too large ! ({} > {})",
            data.len(),
            u32::MAX
        ))
    })?;

    writer.write(&data_len.to_be_bytes())?;
    writer.write(data)?;

    Ok(())
}

pub fn read_message_raw<R: Read>(reader: &mut R) -> Result<Box<[u8]>, ServerError> {
    let mut data_len_buffer = [0u8; 4];

    reader.read_exact(&mut data_len_buffer)?;

    let data_len = u32::from_be_bytes(data_len_buffer);

    if data_len > MAX_MESSAGE_SIZE {
        return ServerError::transmission_error(format!(
            "Received message is too big ! ({} > {})",
            data_len, MAX_MESSAGE_SIZE
        ));
    }

    let mut data = Vec::with_capacity(data_len as usize);
    data.resize(data_len as usize, 0u8);

    reader.read_exact(&mut data.as_mut_slice())?;

    Ok(data.into_boxed_slice())
}

///TODO: Use Read::take() and BufReader instead ?
pub fn read_message<R: Read>(reader: &mut R) -> Result<Message, ServerError> {
    let data = read_message_raw(reader)?;

    let message = serde_json::from_slice::<Message>(&data)?;

    Ok(message)
}
