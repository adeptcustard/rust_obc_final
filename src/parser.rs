use crate::commands::{is_valid_tlv, CommandType};
use crate::errors::PacketError;
use crate::packet::{MAX_PACKET_SIZE, Packet, TLV};

pub fn parse_packet(bytes: &[u8], seq: u32, timestamp: u64) -> Result<Packet, PacketError> {
    if bytes.is_empty() {
        return Err(PacketError::EmptyPacket);
    }

    if bytes.len() < 2 {
        return Err(PacketError::InvalidLength);
    }

    if bytes.len() > MAX_PACKET_SIZE {
        return Err(PacketError::PacketTooLarge);
    }

    let typ = bytes[0];
    let len = bytes[1] as usize;

    if CommandType::from_byte(typ).is_none() {
        return Err(PacketError::UnknownType);
    }

    if bytes.len().saturating_sub(2) != len {
        return Err(PacketError::InvalidLength);
    }

    let val = bytes[2..].to_vec();

    if !is_valid_tlv(typ, len as u16, &val) {
        return Err(PacketError::InvalidValue);
    }

    Ok(Packet {
        timestamp,
        seq,
        tlvs: vec![TLV {
            typ,
            len: len as u16,
            val,
        }],
    })
}

