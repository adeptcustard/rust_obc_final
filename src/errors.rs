#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    EmptyPacket,
    InvalidLength,
    UnknownType,
    InvalidValue,
    PacketTooLarge,
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::EmptyPacket => write!(f, "empty packet"),
            PacketError::InvalidLength => write!(f, "invalid packet length"),
            PacketError::UnknownType => write!(f, "unknown command type"),
            PacketError::InvalidValue => write!(f, "invalid command value"),
            PacketError::PacketTooLarge => write!(f, "packet exceeds maximum size"),
        }
    }
}

impl std::error::Error for PacketError {}

