use crate::state::OBCState;

pub fn update_subsystems(state: &mut OBCState, dt_ms: u64) {
    update_power(state, dt_ms);
    update_temps(state, dt_ms);
    update_comms(state, dt_ms);
    update_payload(state);
    update_adcs(state, dt_ms);
}

pub fn update_power(state: &mut OBCState, dt_ms: u64) {
    let dt_s = dt_ms as f32 / 1000.0;
    state.power.battery_voltage -= 0.0001 * (dt_ms as f32 / 1000.0);
    state.power.solar_input = (state.power.solar_input + 0.1).sin().abs() * 2.0;

    if state.power.solar_input > 0.5 {
        state.power.battery_voltage += 0.0005 * dt_s;
        state.power.charging = true;
    } else {
        state.power.charging = false;
    }

    if state.power.battery_voltage < 3.5 {
        state.power.mode = crate::state::PowerStatus::LowPower;
    } else {
        state.power.mode = crate::state::PowerStatus::Normal;
    }
}

pub fn update_temps(state: &mut OBCState, dt_ms: u64) {
    let delta = 0.01 * (dt_ms as f32 / 1000.0);
    state.temps.obc_board += delta;
    state.temps.radio += delta * 0.8;
    state.temps.payload += delta * 0.5;
    state.temps.battery += delta * 1.1;
}

pub fn update_comms(state: &mut OBCState, dt_ms: u64) {
    state.comms.last_uplink_ms += dt_ms;
    state.comms.last_downlink_ms += dt_ms;
    state.comms.rssi -= 0.05;
}

pub fn update_payload(state: &mut OBCState) {
    if state.payload.payload_on == true {
        state.payload.status = crate::state::PayloadStatus::Busy;
    } else {
        state.payload.status = crate::state::PayloadStatus::Idle;
    }
}

pub fn update_adcs(state: &mut OBCState, dt_ms: u64) {
    let drift = 0.001 * (dt_ms as f32 / 1000.0);
    state.adcs.gyro.0 += drift;
    state.adcs.gyro.1 += drift;
    state.adcs.gyro.2 += drift;

    state.adcs.mag.0 += 0.1;
    state.adcs.mag.1 += 0.1;
    state.adcs.mag.2 += 0.1;
}