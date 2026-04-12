mod packet;
mod errors;
mod commands;
mod modes;
mod parser;
mod handler;
mod logger;
mod faults;
mod tests;
mod state;
mod subsystems;
mod post;
mod storage;

use std::io;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::faults::evaluate_faults;
use crate::handler::handle_packet;
use crate::parser::parse_packet;
use crate::post::run_post;
use crate::state::{OBCMode, OBCState};
use crate::subsystems::update_subsystems;

fn main() {
    let mut state = OBCState::new();
    let mut seq: u32 = 0;
    let mut last_packet = String::new();

    logger::info("[BOOT] Starting OBC runtime");

    if run_post(&state) {
        logger::info("[BOOT] POST passed");
    } else {
        state.mode = OBCMode::Safe;
        logger::warn("[BOOT] POST failed, entering SAFE mode");
    }

    if let Err(e) = storage::initialise_storage() {
        logger::error(&format!("Storage init failed: {e}"));
    }

    let tick_ms: u64 = 1000;
    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        if tx.send(input).is_err() {
            break;
        }
    });

    loop {
        if let Ok(notes) = storage::apply_external_changes(&mut state) {
            for note in notes {
                logger::info_file_only(&note);
            }
        }

        update_subsystems(&mut state, 1000);
        if evaluate_faults(&mut state) {
            logger::warn("Fault detected, SAFE mode asserted");
        }

        logger::info_file_only(&format!(
            "Mode={:?} Vbatt={:.3} RSSI={:.1}",
            state.mode, state.power.battery_voltage, state.comms.rssi
        ));

        let input = match rx.recv_timeout(Duration::from_millis(tick_ms)) {
            Ok(line) => line,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(e) = storage::write_state(&state, &last_packet) {
                    logger::error(&format!("Storage write failed: {e}"));
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                logger::warn("Input channel closed, ending runtime");
                break;
            }
        };

        let trimmed = input.trim();

        if trimmed.is_empty() {
            if let Err(e) = storage::write_state(&state, &last_packet) {
                logger::error(&format!("Storage write failed: {e}"));
            }
            continue;
        }

        let mut bytes = Vec::new();
        let mut valid_tlv_input = true;
        for token in trimmed.split_whitespace() {
            match u8::from_str_radix(token, 16) {
                Ok(value) => bytes.push(value),
                Err(_) => {
                    valid_tlv_input = false;
                    break;
                }
            }
        }

        if !valid_tlv_input {
            println!("[WARN] Rejected input: only TLV hex bytes are accepted");
            logger::warn("Rejected input: only TLV hex bytes are accepted");
            continue;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match parse_packet(&bytes, seq, timestamp) {
            Ok(packet) => {
                logger::info_at(
                    &format!("RX packet seq={} tlv_count={}", packet.seq, packet.tlvs.len()),
                    packet.timestamp,
                );
                last_packet = trimmed.to_string();
                if !handle_packet(packet, &mut state) {
                    logger::info("Runtime shutdown accepted");
                    break;
                }
            }
            Err(err) => {
                println!("[WARN] Rejected invalid command: {err}");
                logger::warn(&format!("Rejected invalid command: {err}"));
            }
        }

        seq = seq.wrapping_add(1);

        if let Err(e) = storage::write_state(&state, &last_packet) {
            logger::error(&format!("Storage write failed: {e}"));
        }
    }
}
