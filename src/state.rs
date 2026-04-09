// OBC State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OBCMode {
    Normal,
    Safe
}
pub struct OBCState {
    pub mode: OBCMode,

    pub power: PowerState,
    pub temps: TempsState,
    pub comms: CommsState,
    pub payload: PayloadState,
    pub adcs: ADCSState,
    pub faults: FaultsState
}

impl OBCState {
    pub fn new() -> Self {
        Self {
            mode: OBCMode::Normal,
            power: PowerState::new(),
            temps: TempsState::new(),
            comms: CommsState::new(),
            payload: PayloadState::new(),
            adcs: ADCSState::new(),
            faults: FaultsState::new()
        }
    }
}

// Power Stats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerStatus {
    Normal,
    LowPower,
    Critical
}
pub struct PowerState{
    pub battery_voltage: f32,
    pub battery_current: f32,
    pub mode: PowerStatus,
    pub charging: bool
}
impl PowerState {
    pub fn new() -> Self {
        Self {
            battery_voltage: 4.0,
            battery_current: 0.0,
            mode: PowerStatus::Normal,
            charging: true
        }
    }
}

// Temps Stats
pub struct TempsState {
    pub obc_board: f32,
    pub radio: f32,
    pub payload: f32,
    pub battery: f32
}
impl TempsState {
    pub fn new() -> Self {
        Self {
            obc_board: 20.0,
            radio: 20.0,
            payload: 20.0,
            battery: 20.0
        }
    }
}

// Comms Stats
pub struct CommsState {
    pub radio_on: bool,
    pub rssi: f32,
    pub last_uplink_ms: u64,
    pub last_downlink_ms: u64
}
impl CommsState {
    pub fn new() -> Self {
        Self {
            radio_on: true,
            rssi: -70.0,
            last_uplink_ms: 0,
            last_downlink_ms: 0
        }
    }
}

// Payload Stats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadStatus{
    Idle,
    Busy,
    Error
}
pub struct PayloadState {
    pub payload_on: bool,
    pub status: PayloadStatus,
}
impl PayloadState {
    pub fn new() -> Self {
        Self {
            payload_on: true,
            status: PayloadStatus::Idle
        }
    }
}

// ADCS Stats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ADCSStatus {
    Detumble,
    Nominal,
    Safe
}
pub struct ADCSState {
    pub status: ADCSStatus,
    pub gyro: (f32, f32, f32),
    pub mag: (f32, f32, f32)
}
impl ADCSState {
    pub fn new() -> Self {
        Self {
            status: ADCSStatus::Detumble,
            gyro: (0.0, 0.0, 0.0),
            mag: (0.0, 0.0, 0.0)
        }
    }
}

// Faults Stats
pub struct FaultsState {
    pub power_fault: bool,
    pub temp_fault: bool,
    pub comms_fault: bool,
    pub payload_fault: bool,
    pub adcs_fault: bool

}
impl FaultsState {
    pub fn new() -> Self {
        Self {
            power_fault: false,
            temp_fault: false,
            comms_fault: false,
            payload_fault: false,
            adcs_fault: false
        }
    }
}