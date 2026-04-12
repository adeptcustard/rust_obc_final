use crate::commands::CommandType;
use crate::faults::{clear_faults, evaluate_faults, has_active_faults, run_fault_test};
use crate::logger;
use crate::modes::valid_transition;
use crate::packet::{Packet, TLV};
use crate::state::{ADCSStatus, OBCMode, OBCState, PayloadStatus};
use crate::storage;

pub fn handle_packet(packet: Packet, state: &mut OBCState) -> bool {
    let packet_ts = packet.timestamp;
    for tlv in packet.tlvs {
        if !handle_tlv(&tlv, state, packet_ts) {
            return false;
        }
    }

    true
}

fn handle_tlv(tlv: &TLV, state: &mut OBCState, packet_ts: u64) -> bool {
    let command = match CommandType::from_byte(tlv.typ) {
        Some(cmd) => cmd,
        None => {
            logger::warn_at("Rejected invalid command type", packet_ts);
            return true;
        }
    };

    match command {
        CommandType::System => handle_system_command(tlv, state, packet_ts),
        CommandType::Telemetry => {
            handle_telemetry_command(tlv, state, packet_ts);
            true
        }
        CommandType::Comms => {
            handle_comms_command(tlv, state, packet_ts);
            true
        }
        CommandType::Payload => handle_payload_command(tlv, state, packet_ts),
        CommandType::Adcs => {
            handle_adcs_command(tlv, state, packet_ts);
            true
        }
        CommandType::FaultManagement => {
            handle_fault_management_command(tlv, state, packet_ts);
            true
        }
        CommandType::FaultInjection => {
            handle_fault_injection_command(tlv, state, packet_ts);
            true
        }
        CommandType::Storage => {
            handle_storage_command(tlv, packet_ts);
            true
        }
    }
}

fn require_single_value_len(tlv: &TLV, label: &str, packet_ts: u64) -> bool {
    if tlv.len != 1 || tlv.val.len() != 1 {
        logger::error_at(&format!("{label} command requires length 0x01"), packet_ts);
        return false;
    }

    true
}

fn handle_system_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) -> bool {
    match tlv.len {
        0 => {
            if !tlv.val.is_empty() {
                logger::error_at("System command length/value mismatch", packet_ts);
                return true;
            }

            logger::warn_at("Shutdown requested via 01 00", packet_ts);
            return false;
        }
        1 => {}
        _ => {
            logger::error_at("System command invalid length", packet_ts);
            return true;
        }
    }

    if tlv.val.len() != 1 {
        logger::error_at("System command length/value mismatch", packet_ts);
        return true;
    }

    let requested_mode = match tlv.val[0] {
        0x00 => OBCMode::Normal,
        0x01 => OBCMode::Safe,
        _ => {
            logger::error_at("Invalid mode value", packet_ts);
            return true;
        }
    };

    if valid_transition(state.mode, requested_mode, has_active_faults(state)) {
        state.mode = requested_mode;
        logger::info_at("Mode changed successfully", packet_ts);
    } else {
        logger::warn_at("Invalid mode transition blocked", packet_ts);
    }

    true
}

fn handle_telemetry_command(tlv: &TLV, state: &OBCState, packet_ts: u64) {
    if !require_single_value_len(tlv, "Telemetry", packet_ts) {
        return;
    }

    match tlv.val[0] {
        0x00 => {
            logger::info_at(&format!(
                "FULL | mode={:?} vbatt={:.3} ibatt={:.3} temp_obc={:.2} rssi={:.1} payload={:?} adcs={:?}",
                state.mode,
                state.power.battery_voltage,
                state.power.battery_current,
                state.temps.obc_board,
                state.comms.rssi,
                state.payload.status,
                state.adcs.status
            ), packet_ts);
        }
        0x01 => logger::info_at(&format!(
            "POWER | vbatt={:.3} ibatt={:.3} pwr_mode={:?} charging={} solar={:.3}",
            state.power.battery_voltage,
            state.power.battery_current,
            state.power.mode,
            state.power.charging,
            state.power.solar_input
        ), packet_ts),
        0x02 => logger::info_at(&format!(
            "TEMPS | obc={:.2} radio={:.2} payload={:.2} battery={:.2}",
            state.temps.obc_board, state.temps.radio, state.temps.payload, state.temps.battery
        ), packet_ts),
        0x03 => logger::info_at(&format!(
            "COMMS | radio_on={} rssi={:.1} uplink_ms={} downlink_ms={}",
            state.comms.radio_on, state.comms.rssi, state.comms.last_uplink_ms, state.comms.last_downlink_ms
        ), packet_ts),
        0x04 => logger::info_at(&format!(
            "PAYLOAD | on={} status={:?}",
            state.payload.payload_on, state.payload.status
        ), packet_ts),
        0x05 => logger::info_at(&format!(
            "ADCS | status={:?} gyro=({:.3},{:.3},{:.3}) mag=({:.3},{:.3},{:.3})",
            state.adcs.status,
            state.adcs.gyro.0,
            state.adcs.gyro.1,
            state.adcs.gyro.2,
            state.adcs.mag.0,
            state.adcs.mag.1,
            state.adcs.mag.2
        ), packet_ts),
        0x06 => match logger::dump_error_logs() {
            Ok(count) => logger::info_at(&format!("Dumped {count} warning/error lines to error_dump.txt"), packet_ts),
            Err(e) => logger::error_at(&format!("Failed to dump error logs: {e}"), packet_ts),
        },
        _ => logger::error_at("Invalid telemetry value", packet_ts),
    }
}

fn handle_comms_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) {
    if !require_single_value_len(tlv, "Comms", packet_ts) {
        return;
    }

    match tlv.val[0] {
        0x00 => {
            state.comms.radio_on = false;
            logger::warn_at("Radio disabled", packet_ts);
        }
        0x01 => {
            state.comms.radio_on = true;
            logger::info_at("Radio enabled", packet_ts);
        }
        0x02 => logger::info_at("Comms switched to transmit mode", packet_ts),
        0x03 => logger::info_at("Comms switched to receive mode", packet_ts),
        0x04 => {
            state.comms.radio_on = true;
            state.comms.rssi = -70.0;
            state.comms.last_uplink_ms = 0;
            state.comms.last_downlink_ms = 0;
            logger::info_at("Radio reset complete", packet_ts);
        }
        _ => logger::error_at("Invalid comms value", packet_ts),
    }
}

fn handle_payload_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) -> bool {
    if !require_single_value_len(tlv, "Payload", packet_ts) {
        return true;
    }

    match tlv.val[0] {
        0x00 => {
            state.payload.payload_on = false;
            state.payload.status = PayloadStatus::Idle;
            logger::warn_at("Payload disabled", packet_ts);
        }
        0x01 => {
            state.payload.payload_on = true;
            state.payload.status = PayloadStatus::Idle;
            logger::info_at("Payload enabled", packet_ts);
        }
        0x02 => {
            state.payload.payload_on = true;
            state.payload.status = PayloadStatus::Busy;
            logger::info_at("Payload capture started", packet_ts);
        }
        0x03 => {
            state.payload.status = PayloadStatus::Idle;
            logger::info_at("Payload capture stopped", packet_ts);
        }
        0x04 => logger::info_at("Payload buffer clear requested", packet_ts),
        _ => logger::error_at("Invalid payload value", packet_ts),
    }

    true
}

fn handle_adcs_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) {
    if !require_single_value_len(tlv, "ADCS", packet_ts) {
        return;
    }

    match tlv.val[0] {
        0x00 => {
            state.adcs.status = ADCSStatus::Safe;
            logger::warn_at("ADCS disabled", packet_ts);
        }
        0x01 => {
            state.adcs.status = ADCSStatus::Nominal;
            logger::info_at("ADCS enabled", packet_ts);
        }
        0x02 => {
            state.adcs.status = ADCSStatus::Detumble;
            logger::info_at("ADCS detumble mode set", packet_ts);
        }
        0x03 => {
            state.adcs.status = ADCSStatus::Nominal;
            logger::info_at("ADCS sun-pointing target set", packet_ts);
        }
        0x04 => {
            state.adcs.status = ADCSStatus::Nominal;
            logger::info_at("ADCS nadir-pointing target set", packet_ts);
        }
        0x05 => {
            state.adcs.status = ADCSStatus::Safe;
            logger::warn_at("ADCS safe pointing set", packet_ts);
        }
        _ => logger::error_at("Invalid ADCS value", packet_ts),
    }
}

fn handle_fault_management_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) {
    if !require_single_value_len(tlv, "Fault management", packet_ts) {
        return;
    }

    match tlv.val[0] {
        0x00 => {
            clear_faults(state);
            logger::info_at("All faults cleared", packet_ts);
        }
        0x01 => {
            let mode = state.mode;
            *state = OBCState::new();
            state.mode = mode;
            logger::info_at("All subsystems reset", packet_ts);
        }
        0x02 => {
            state.comms.radio_on = true;
            state.comms.rssi = -70.0;
            state.comms.last_uplink_ms = 0;
            state.comms.last_downlink_ms = 0;
            state.faults.comms_fault = false;
            logger::info_at("Comms reset", packet_ts);
        }
        0x03 => {
            state.payload.payload_on = true;
            state.payload.status = PayloadStatus::Idle;
            state.faults.payload_fault = false;
            logger::info_at("Payload reset", packet_ts);
        }
        0x04 => {
            state.adcs.status = ADCSStatus::Detumble;
            state.adcs.gyro = (0.0, 0.0, 0.0);
            state.adcs.mag = (0.0, 0.0, 0.0);
            state.faults.adcs_fault = false;
            logger::info_at("ADCS reset", packet_ts);
        }
        _ => logger::error_at("Invalid fault management value", packet_ts),
    }
}

fn handle_fault_injection_command(tlv: &TLV, state: &mut OBCState, packet_ts: u64) {
    if !require_single_value_len(tlv, "Fault injection", packet_ts) {
        return;
    }

    run_fault_test(state, tlv.val[0]);
    if evaluate_faults(state) {
        logger::warn_at("Fault condition active: SAFE mode enforced", packet_ts);
    }
    logger::warn_at("Fault injection executed", packet_ts);
}

fn handle_storage_command(tlv: &TLV, packet_ts: u64) {
    match tlv.len {
        0 => match storage::dump_storage() {
            Ok(contents) => logger::info_at(&format!("STORAGE DUMP\n{contents}"), packet_ts),
            Err(e) => logger::error_at(&format!("Storage dump failed: {e}"), packet_ts),
        },
        1 => {
            if tlv.val.len() != 1 {
                logger::error_at("Storage command length/value mismatch", packet_ts);
                return;
            }

            match tlv.val[0] {
            0x00 => match storage::clear_storage() {
                Ok(()) => logger::warn_at("Storage cleared", packet_ts),
                Err(e) => logger::error_at(&format!("Storage clear failed: {e}"), packet_ts),
            },
            0x01 => match storage::write_test_data() {
                Ok(()) => logger::info_at("Storage test data written", packet_ts),
                Err(e) => logger::error_at(&format!("Storage test write failed: {e}"), packet_ts),
            },
            0x02 => match storage::corrupt_storage() {
                Ok(()) => logger::warn_at("Storage deliberately corrupted", packet_ts),
                Err(e) => logger::error_at(&format!("Storage corruption failed: {e}"), packet_ts),
            },
            0x03 => match storage::restore_defaults() {
                Ok(()) => logger::info_at("Storage restored to defaults", packet_ts),
                Err(e) => logger::error_at(&format!("Storage restore failed: {e}"), packet_ts),
            },
            _ => logger::error_at("Invalid storage value", packet_ts),
        }
        }
        _ => logger::error_at("Storage command invalid length", packet_ts),
    }
}

