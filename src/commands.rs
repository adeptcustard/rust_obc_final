#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    System,
    Telemetry,
    Comms,
    Payload,
    Adcs,
    FaultManagement,
    FaultInjection,
    Storage,
}

impl CommandType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(CommandType::System),
            0x02 => Some(CommandType::Telemetry),
            0x03 => Some(CommandType::Comms),
            0x04 => Some(CommandType::Payload),
            0x05 => Some(CommandType::Adcs),
            0x06 => Some(CommandType::FaultManagement),
            0x07 => Some(CommandType::FaultInjection),
            0x08 => Some(CommandType::Storage),
            _ => None,
        }
    }
}

pub fn is_valid_tlv(typ: u8, len: u16, val: &[u8]) -> bool {
    if val.len() != usize::from(len) {
        return false;
    }

    match typ {
        // 01 00 (shutdown) or 01 01 xx (mode switch)
        0x01 => match len {
            0 => val.is_empty(),
            1 => matches!(val, [0x00] | [0x01]),
            _ => false,
        },
        0x02 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04] | [0x05] | [0x06]),
        0x03 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04]),
        0x04 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04]),
        0x05 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04] | [0x05]),
        0x06 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04]),
        0x07 => len == 1 && matches!(val, [0x00] | [0x01] | [0x02] | [0x03] | [0x04] | [0x05] | [0x06] | [0x07]),
        // 08 00 (dump) or 08 01 xx (storage action)
        0x08 => match len {
            0 => val.is_empty(),
            1 => matches!(val, [0x00] | [0x01] | [0x02] | [0x03]),
            _ => false,
        },
        _ => false,
    }
}

