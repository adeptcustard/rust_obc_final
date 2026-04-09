use crate::state::{OBCState, PayloadStatus};

pub fn run_post(state: &OBCState) -> bool {
    let power_ok =
        state.power.battery_voltage > 3.0 &&
        state.power.battery_current < 2.0 &&
        state.power.mode != crate::state::PowerStatus::Critical;

    let temps_ok =
        state.temps.obc_board < 85.0 &&
        state.temps.radio < 85.0 &&
        state.temps.payload < 85.0 &&
        state.temps.battery < 60.0;

    let comms_ok =
        state.comms.radio_on &&
        state.comms.rssi > -110.0 &&
        state.comms.last_uplink_ms < 30_000 &&
        state.comms.last_downlink_ms < 30_000;

    let payload_ok =
        state.payload.payload_on &&
        state.payload.status != PayloadStatus::Error;

    let adcs_ok =
        state.adcs.gyro.0.abs() < 0.1 &&
        state.adcs.gyro.1.abs() < 0.1 &&
        state.adcs.gyro.2.abs() < 0.1 &&
        state.adcs.mag.0.abs() < 0.1 &&
        state.adcs.mag.1.abs() < 0.1 &&
        state.adcs.mag.2.abs() < 0.1 &&
        state.adcs.status != crate::state::ADCSStatus::Safe;

    let faults_present =
        !state.faults.power_fault &&
        !state.faults.temp_fault &&
        !state.faults.comms_fault &&
        !state.faults.payload_fault &&
        !state.faults.adcs_fault;

    power_ok && temps_ok && comms_ok && payload_ok && adcs_ok && faults_present
}