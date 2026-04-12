pub struct Packet {
    pub timestamp: u64,
    pub seq: u32,
    pub tlvs: Vec<TLV>
}

pub struct TLV {
    pub typ: u8,
    pub len: u16,
    pub val: Vec<u8>
}

pub const MAX_PACKET_SIZE: usize = 32;