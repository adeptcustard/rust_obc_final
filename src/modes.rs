use crate::state::{OBCMode, OBCState};

pub fn valid_transition(current: OBCMode, next: OBCMode, faults_active: bool) -> bool {
    match (current, next) {
        (_, OBCMode::Safe) => true,
        (OBCMode::Safe, OBCMode::Normal) => !faults_active,
        _ => true,
    }
}

pub fn force_safe_mode(state: &mut OBCState) {
    state.mode = OBCMode::Safe;
}

