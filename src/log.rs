use crate::consts::{BRUTEFORCE_LOG_FORMATTED, BRUTEFORCE_LOG_HUMAN, BRUTEFORCE_POTENTIAL_PERFECTS_FORMATTED, BRUTEFORCE_POTENTIAL_PERFECTS_HUMAN};
use crate::helpers::keep_last_n_lines;
use crate::{consts, ATTEMPTS, CUR_NSECS, CUR_SECS, PERFECTS, START_ATTEMPTS, START_PERFECTS, START_TIME};
use chrono::{Datelike, Local, Timelike};
use colored::Colorize;
use hhmmss::Hhmmss;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;

#[derive(PartialEq, Clone, Copy)]
pub enum LogType {
    FailedRun,
    Ooh,
    Close,
    PotentiallyPerfectRun,
    Round4PerfectRun,
}

fn format_log_timestamp() -> String {
    let dt = Local::now();
    let ts = format!(
        "{:04?}-{:02?}-{:02?} {:02?}:{:02?}:{:02?}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    );
    ts
}

fn format_session_duration() -> String {
    START_TIME.load(Ordering::SeqCst).elapsed().hhmmssxxx()
}

fn format_messages(log_type: LogType,
   perfect_runs: i32,
   attempts: i32,
   secs: i32,
   nsecs: i32,
   perfect_fruits: (i32, String),
   perfect_patterns: (i32, String),
   color_hex: &str,
) -> (String , String) {
    let heading = match log_type {
        LogType::FailedRun => "Failed! ",
        LogType::Close => "Close!  ",
        LogType::Ooh => "Ooh.",
        LogType::PotentiallyPerfectRun => "Perfect!",
        LogType::Round4PerfectRun => "Round 4 Perfect Run!",
    };
    let session_perfects = format!("{:06}", PERFECTS.load(Ordering::SeqCst) - START_PERFECTS.load(Ordering::SeqCst));
    let session_attempts = format!("{:06}", ATTEMPTS.load(Ordering::SeqCst) - START_ATTEMPTS.load(Ordering::SeqCst));
    let _unformatted = format!("{:04}/{:07} [+{:06}/{:06}] | {:03?}s {:09?}ns | {} | {} | ",
        perfect_runs,
        attempts,
        session_perfects,
        session_attempts,
        secs,
        nsecs,
        format_log_timestamp(),
        format_session_duration(),
    );
    let mut part2 = format!("{} | F: {}/5 | P: {}/5 | R1 P: {} | R1 FRT: {}",
        heading,
        perfect_fruits.0,
        perfect_patterns.0,
        perfect_patterns.1,
        perfect_fruits.1,
    );
    match log_type {
        LogType::Ooh => {
            part2 = "Ooh".to_owned();
        },
        _ => {},
    }
    (format!("{}{}", _unformatted, part2.color(color_hex)), format!("{}{}", _unformatted, part2))
}

fn write_and_flush(filename: &str, msg: &str) {
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(filename)
        .unwrap();
    writeln!(&mut file, "{}", msg)
        .unwrap();
    match file.flush() {
        Ok(_) => { println!("flushed"); },
        Err(e) => { println!("flush failed {}", e); },
    };
}

pub fn write_log(filename: &str, filename_human: &str, msg: &str, msg_human: &str) {
    if ATTEMPTS.load(Ordering::SeqCst) % consts::LOG_FREQUENCY == 0 {
        keep_last_n_lines(filename, 20).unwrap();
        keep_last_n_lines(filename_human, 20).unwrap();
    } 
    write_and_flush(filename, msg);
    write_and_flush(filename_human, msg_human);
}

pub fn log(log_type: LogType, perfect_runs: i32, attempts: i32, secs: i32, nsecs:
i32, perfect_fruits: (i32, String), perfect_patterns: (i32, String)) {
    let mut _msg = "".to_owned();
    let mut _msg_human = "".to_owned();
    let color_hex = match log_type {
        LogType::FailedRun => "#FF0000",
        LogType::Close => "#00E1FF",
        LogType::Ooh => "#0000FF",
        LogType::PotentiallyPerfectRun => "#FFD700",
        LogType::Round4PerfectRun => "#00E1FF"
    };
    (_msg, _msg_human) = format_messages(log_type,
        perfect_runs,
        attempts,
        secs,
        nsecs,
        perfect_fruits.clone(),
        perfect_patterns.clone(),
        color_hex
    );
    if log_type == LogType::PotentiallyPerfectRun {
        PERFECTS.fetch_add(1, Ordering::SeqCst);
        write_log(BRUTEFORCE_POTENTIAL_PERFECTS_FORMATTED, BRUTEFORCE_POTENTIAL_PERFECTS_HUMAN, _msg.as_str(), _msg_human.as_str());
    }
    write_log(BRUTEFORCE_LOG_FORMATTED, BRUTEFORCE_LOG_HUMAN, _msg.as_str(), _msg_human.as_str());
}

fn lines_from_file(file: String) -> String {
    let file: Vec<String> = file.lines().map(|l| l.to_owned()).collect();
    file.last().unwrap().to_owned()
}

pub fn init_read() {
    let line = lines_from_file(std::fs::read_to_string(consts::BRUTEFORCE_LOG_HUMAN).unwrap());
    let perfect_runs = line[0..=3].parse::<i32>().unwrap();
    let attempts = line[5..=11].parse::<i32>().unwrap();
    let secs = line[32..=34].parse::<i32>().unwrap();
    let nsecs = line[37..=45].parse::<i32>().unwrap();
    PERFECTS.store(perfect_runs, Ordering::SeqCst);
    START_PERFECTS.store(perfect_runs, Ordering::SeqCst);
    START_ATTEMPTS.store(attempts, Ordering::SeqCst);
    ATTEMPTS.store(attempts + 1, Ordering::SeqCst);
    CUR_SECS.store(secs, Ordering::SeqCst);
    CUR_NSECS.store(nsecs + 1000, Ordering::SeqCst);
}
