#[cfg(test)]
mod tests {
    use crate::errors::PacketError;
    use crate::modes::valid_transition;
    use crate::parser::parse_packet;
    use crate::state::OBCMode;

    #[test]
    fn parse_valid_system_mode_packet() {
        let bytes = [0x01, 0x01, 0x01];
        let packet = parse_packet(&bytes, 7, 1234).expect("packet should parse");
        assert_eq!(packet.seq, 7);
        assert_eq!(packet.tlvs.len(), 1);
        assert_eq!(packet.tlvs[0].typ, 0x01);
    }

    #[test]
    fn parse_valid_shutdown_packet() {
        let bytes = [0x01, 0x00];
        let packet = parse_packet(&bytes, 3, 99).expect("shutdown packet should parse");
        assert_eq!(packet.tlvs[0].len, 0);
        assert!(packet.tlvs[0].val.is_empty());
    }

    #[test]
    fn parse_rejects_unknown_type() {
        let bytes = [0x99, 0x01, 0x00];
        match parse_packet(&bytes, 0, 0) {
            Err(err) => assert_eq!(err, PacketError::UnknownType),
            Ok(_) => panic!("unknown command must fail"),
        }
    }

    #[test]
    fn safe_to_normal_blocked_when_faults_active() {
        let allowed = valid_transition(OBCMode::Safe, OBCMode::Normal, true);
        assert!(!allowed);
    }

    #[test]
    fn parse_rejects_invalid_value_for_command() {
        let bytes = [0x08, 0x01, 0xFF];
        match parse_packet(&bytes, 0, 0) {
            Err(err) => assert_eq!(err, PacketError::InvalidValue),
            Ok(_) => panic!("invalid value must fail"),
        }
    }

    #[test]
    fn parse_rejects_system_wrong_length() {
        let bytes = [0x01, 0x02, 0x00, 0x01];
        match parse_packet(&bytes, 0, 0) {
            Err(err) => assert_eq!(err, PacketError::InvalidValue),
            Ok(_) => panic!("system command length must follow COMMANDS.txt"),
        }
    }

    #[test]
    fn parse_rejects_storage_length_one_without_value() {
        let bytes = [0x08, 0x01];
        match parse_packet(&bytes, 0, 0) {
            Err(err) => assert_eq!(err, PacketError::InvalidLength),
            Ok(_) => panic!("storage command length/value mismatch must fail"),
        }
    }
}


