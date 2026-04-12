use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Local, TimeZone};

pub const LOG_FILE: &str = "obc_runtime.log";

fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn format_timestamp(ts_millis: u64) -> String {
    match Local.timestamp_millis_opt(ts_millis as i64).single() {
        Some(dt) => dt.format("%d/%m/%Y_%H:%M:%S").to_string(),
        None => "01/01/1970_00:00:00".to_string(),
    }
}

pub fn log(level: &str, message: &str) {
    let ts = format_timestamp(timestamp_millis());
    let line = format!("[{ts}] [{level}] {message}");
    write_to_file(&line);
}

pub fn log_file_only(level: &str, message: &str) {
    let ts = format_timestamp(timestamp_millis());
    let line = format!("[{ts}] [{level}] {message}");
    write_to_file(&line);
}

pub fn log_at(level: &str, message: &str, ts_millis: u64) {
    let ts = format_timestamp(ts_millis);
    let line = format!("[{ts}] [{level}] {message}");
    println!("[{level}] {message}");
    write_to_file(&line);
}

fn write_to_file(line: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = writeln!(file, "{line}");
    }
}

pub fn read_all_logs() -> std::io::Result<String> {
    let mut file = File::open(LOG_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn dump_error_logs() -> std::io::Result<usize> {
    let logs = read_all_logs()?;
    let error_lines: Vec<&str> = logs
        .lines()
        .filter(|line| line.contains("[ERROR]") || line.contains("[WARN]"))
        .collect();

    let mut file = File::create("error_dump.txt")?;
    for line in &error_lines {
        writeln!(file, "{line}")?;
    }

    Ok(error_lines.len())
}

pub fn info(message: &str) {
    log("INFO", message);
}

pub fn info_at(message: &str, ts_millis: u64) {
    log_at("INFO", message, ts_millis);
}

pub fn info_file_only(message: &str) {
    log_file_only("INFO", message);
}

pub fn warn(message: &str) {
    log("WARN", message);
}

pub fn warn_at(message: &str, ts_millis: u64) {
    log_at("WARN", message, ts_millis);
}

pub fn error(message: &str) {
    log("ERROR", message);
}

pub fn error_at(message: &str, ts_millis: u64) {
    log_at("ERROR", message, ts_millis);
}


