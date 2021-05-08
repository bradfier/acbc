use crate::protocol::PROTOCOL_VERSION;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

fn write_kstring<W: Write>(string: &str, writer: &mut W) -> std::io::Result<()> {
    let bytes = string.as_bytes();
    assert!(bytes.len() < u16::MAX as usize);
    writer.write_u16::<LittleEndian>(bytes.len() as u16)?;
    writer.write_all(bytes)
}

pub trait OutboundMessage<W> {
    fn encode(self, writer: &mut W) -> std::io::Result<()>;
}

#[derive(Debug, Clone)]
pub struct RegistrationRequest<'a> {
    version: u8,
    username: &'a str,
    password: &'a str,
    interval: u32,
    command_password: &'a str,
}

impl<'a> RegistrationRequest<'a> {
    pub fn new(
        username: &'a str,
        password: &'a str,
        interval: u32,
        command_password: &'a str,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            username,
            password,
            interval,
            command_password,
        }
    }
}

impl<W: Write> OutboundMessage<W> for RegistrationRequest<'_> {
    fn encode(self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[0x01])?; // Packet type
        writer.write_u8(self.version)?; // Protocol version header
        write_kstring(&self.username, writer)?;
        write_kstring(&self.password, writer)?;
        writer.write_u32::<LittleEndian>(self.interval)?;
        write_kstring(&self.command_password, writer)
    }
}

#[derive(Debug, Clone)]
pub struct UnregisterRequest {
    connection_id: u32,
}

impl UnregisterRequest {
    pub fn new(connection_id: u32) -> Self {
        Self { connection_id }
    }
}

impl<W: Write> OutboundMessage<W> for UnregisterRequest {
    fn encode(self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[0x09])?; // Packet type
        writer.write_u32::<LittleEndian>(self.connection_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_registration_request() {
        let req = RegistrationRequest::new("Your name", "asd", 250, "");
        let mut buf = vec![];
        req.encode(&mut buf).expect("Failed to encode");

        let expected = &[
            0x01, 0x04, 0x09, 0x00, 0x59, 0x6f, 0x75, 0x72, 0x20, 0x6e, 0x61, 0x6d, 0x65, 0x03,
            0x00, 0x61, 0x73, 0x64, 0xfa, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(&buf, &expected);
    }
}
