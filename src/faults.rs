use crate::modes::force_safe_mode;
use crate::logger;
use crate::state::{OBCState, PayloadStatus, PowerStatus};

pub fn has_active_faults(state: &OBCState) -> bool {
    state.faults.power_fault
        || state.faults.temp_fault
        || state.faults.comms_fault
        || state.faults.payload_fault
        || state.faults.adcs_fault
}

pub fn clear_faults(state: &mut OBCState) {
    state.faults.power_fault = false;
    state.faults.temp_fault = false;
    state.faults.comms_fault = false;
    state.faults.payload_fault = false;
    state.faults.adcs_fault = false;
}

pub fn evaluate_faults(state: &mut OBCState) -> bool {
    let power_fault_now = state.power.battery_voltage < 3.3 || state.power.mode == PowerStatus::Critical;

    let temp_fault_now = state.temps.obc_board > 90.0
        || state.temps.radio > 90.0
        || state.temps.payload > 90.0
        || state.temps.battery > 65.0;

    let comms_fault_now = !state.comms.radio_on || state.comms.rssi < -115.0;

    let payload_fault_now = state.payload.status == PayloadStatus::Error;

    let adcs_fault_now = state.adcs.gyro.0.abs() > 5.0
        || state.adcs.gyro.1.abs() > 5.0
        || state.adcs.gyro.2.abs() > 5.0;

    // Latch injected faults until fault-management explicitly clears/resets them.
    state.faults.power_fault = state.faults.power_fault || power_fault_now;
    state.faults.temp_fault = state.faults.temp_fault || temp_fault_now;
    state.faults.comms_fault = state.faults.comms_fault || comms_fault_now;
    state.faults.payload_fault = state.faults.payload_fault || payload_fault_now;
    state.faults.adcs_fault = state.faults.adcs_fault || adcs_fault_now;

    if has_active_faults(state) {
        force_safe_mode(state);
        return true;
    }

    false
}

pub fn run_fault_test(state: &mut OBCState, id: u8) {
    match id {
        0x00 => {
            let mut packet = vec![0x01, 0x01, 0x00];
            packet[1] ^= 0b0000_0001;
            state.faults.comms_fault = true;
            logger::warn(&format!("Bit flip simulation: {:?}", packet));
        }
        0x01 => {
            state.power.battery_voltage = -1.0;
            state.faults.power_fault = true;
            logger::warn("Memory corruption simulation applied to battery_voltage");
        }
        0x02 => {
            let data = [1_u8, 2, 3];
            state.payload.status = PayloadStatus::Error;
            state.faults.payload_fault = true;
            logger::warn(&format!("Buffer overflow test result: {:?}", data.get(10)));
        }
        0x03 => {
            state.adcs.gyro = (6.0, 0.0, 0.0);
            state.faults.adcs_fault = true;
            logger::warn("Race condition test simulated (single-threaded runtime)");
        }
        0x04 => {
            let stack = [0_u8; 8];
            state.temps.obc_board = 95.0;
            state.faults.temp_fault = true;
            logger::warn(&format!("Stack corruption test guarded, first byte={}", stack[0]));
        }
        0x05 => {
            let maybe_value: Option<u8> = None;
            if maybe_value.is_none() {
                state.comms.radio_on = false;
                state.faults.comms_fault = true;
                logger::warn("Null pointer test prevented safely");
            }
        }
        0x06 => {
            logger::warn("Subsystem failure injected");
            state.faults.payload_fault = true;
            state.faults.comms_fault = true;
        }
        0x07 => {
            logger::warn("Packet corruption injected");
            state.faults.comms_fault = true;
        }
        _ => {
            logger::warn("Unknown fault test requested");
        }
    }
}

