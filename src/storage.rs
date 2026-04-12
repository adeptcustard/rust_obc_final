use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::faults::clear_faults;
use crate::state::{OBCMode, OBCState, PayloadStatus};

pub const STORAGE_FILE: &str = "obc_memory.txt";

pub fn initialise_storage() -> std::io::Result<()> {
    if std::path::Path::new(STORAGE_FILE).exists() {
        return Ok(());
    }

    restore_defaults()
}

pub fn restore_defaults() -> std::io::Result<()> {
    let mut file = File::create(STORAGE_FILE)?;
    writeln!(file, "MODE=NORMAL")?;
    writeln!(file, "BATTERY=OK")?;
    writeln!(file, "COMMS=OK")?;
    writeln!(file, "PAYLOAD=OK")?;
    writeln!(file, "LAST_PACKET=")?;
    writeln!(file, "FAULT=NONE")?;
    Ok(())
}

pub fn dump_storage() -> std::io::Result<String> {
    let mut file = File::open(STORAGE_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn clear_storage() -> std::io::Result<()> {
    let _ = File::create(STORAGE_FILE)?;
    Ok(())
}

pub fn write_test_data() -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(STORAGE_FILE)?;
    writeln!(file, "TEST_COUNTER=1")?;
    writeln!(file, "TEST_DATA=AA BB CC")?;
    Ok(())
}

pub fn corrupt_storage() -> std::io::Result<()> {
    let mut file = File::create(STORAGE_FILE)?;
    writeln!(file, "MODE=GARBAGE")?;
    writeln!(file, "BATTERY=???")?;
    writeln!(file, "CORRUPT=FF FF FF")?;
    Ok(())
}

pub fn write_state(state: &OBCState, last_packet: &str) -> std::io::Result<()> {
    let mut file = File::create(STORAGE_FILE)?;

    let mode = match state.mode {
        OBCMode::Normal => "NORMAL",
        OBCMode::Safe => "SAFE",
    };

    writeln!(file, "MODE={mode}")?;
    writeln!(
        file,
        "BATTERY={}",
        if state.power.battery_voltage >= 3.3 { "OK" } else { "FAIL" }
    )?;
    writeln!(file, "COMMS={}", if state.comms.radio_on { "OK" } else { "FAIL" })?;
    writeln!(
        file,
        "PAYLOAD={}",
        if state.payload.status == PayloadStatus::Error {
            "FAIL"
        } else {
            "OK"
        }
    )?;
    writeln!(file, "LAST_PACKET={last_packet}")?;
    writeln!(
        file,
        "FAULT={}",
        if crate::faults::has_active_faults(state) {
            "ACTIVE"
        } else {
            "NONE"
        }
    )?;

    Ok(())
}

pub fn apply_external_changes(state: &mut OBCState) -> std::io::Result<Vec<String>> {
    let mut file = OpenOptions::new().read(true).open(STORAGE_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut notes = Vec::new();

    for line in contents.lines() {
        let mut split = line.splitn(2, '=');
        let key = split.next().unwrap_or("").trim();
        let value = split.next().unwrap_or("").trim();

        match (key, value) {
            ("MODE", "NORMAL") => {
                state.mode = OBCMode::Normal;
                notes.push("Applied MODE=NORMAL".to_string());
            }
            ("MODE", "SAFE") => {
                state.mode = OBCMode::Safe;
                notes.push("Applied MODE=SAFE".to_string());
            }
            ("BATTERY", "FAIL") => {
                state.power.battery_voltage = 3.2;
                notes.push("Applied BATTERY=FAIL".to_string());
            }
            ("BATTERY", "OK") => {
                if state.power.battery_voltage < 3.7 {
                    state.power.battery_voltage = 3.9;
                }
                notes.push("Applied BATTERY=OK".to_string());
            }
            ("COMMS", "FAIL") => {
                state.comms.radio_on = false;
                notes.push("Applied COMMS=FAIL".to_string());
            }
            ("COMMS", "OK") => {
                state.comms.radio_on = true;
                notes.push("Applied COMMS=OK".to_string());
            }
            ("PAYLOAD", "FAIL") => {
                state.payload.status = PayloadStatus::Error;
                notes.push("Applied PAYLOAD=FAIL".to_string());
            }
            ("PAYLOAD", "OK") => {
                state.payload.status = PayloadStatus::Idle;
                notes.push("Applied PAYLOAD=OK".to_string());
            }
            ("FAULT", "NONE") => {
                clear_faults(state);
                notes.push("Applied FAULT=NONE".to_string());
            }
            ("LAST_PACKET", _) => {
                if !value.is_empty() {
                    notes.push(format!("External LAST_PACKET observed: {value}"));
                }
            }
            _ => {
                if !key.is_empty() {
                    notes.push(format!("Ignored invalid storage key/value: {line}"));
                }
            }
        }
    }

    Ok(notes)
}


