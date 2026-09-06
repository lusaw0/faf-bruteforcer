use crate::log::{log, BruteforceLogType};
use crate::test::determine_run_test;
use crate::{ATTEMPTS, CUR_NSECS, CUR_SECS, PERFECTS, TRUE_PERFECTS, consts};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::Ordering;
use std::path;
use rev_buf_reader::RevBufReader;

pub enum Mode {
    Bruteforcing,
    Testing,
}

pub enum RunStartMode {
    Hardcoded,
    Log,
}

// Global tracker for the currently-running libTAS PID. Stored as an Option<u32>
// inside a Mutex so kill_libtas can read/clear it safely from other threads.
static LIBTAS_PID: OnceLock<Mutex<Option<u32>>> = OnceLock::new();

pub fn perfect_fruits(output: String) -> (i32, String) {
    let mut perfect_fruits = 0;
    let mut fruit = "N/A".to_owned();
    let mut perfect = false;
    for line in output.lines() {
        if line.contains("suboptimal") {
            return (999, "Ooh".to_owned());
        }
        if line.starts_with("[FRU]") {
            for num in 1..=5 {
                if line.contains(&format!("Round {:?}", num)) && line.contains("GOOD") {
                    if num == 1 && !perfect {
                        fruit = line.to_owned().split_off(28);
                        perfect = true;
                    }
                    perfect_fruits += 1;
                }
            }
        }
    }
    (perfect_fruits, fruit)
}
pub fn perfect_patterns(output: String) -> (i32, String) {
    let mut perfect_patterns = 0;
    let mut pattern = "N/A      ".to_owned();
    for line in output.lines() {
        if line.starts_with("[PAT]") {
            let round = line[12..13].to_owned().parse::<i32>().unwrap();
            let pat = line[24..25].to_owned().parse::<i32>().unwrap();
            match round {
                4 => {
                    if pat == 0 || pat == 3 {
                        perfect_patterns += 1;
                    }
                },
                1 | 2 | 3 | 5 => {
                    match pat {
                        0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                            perfect_patterns += 1;
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
            pattern = format!("Pattern {:?}", pat);
        }
    }
    (perfect_patterns, pattern)
}

pub fn perfect_voicelines(output: String) -> i32 {
    let mut perfect_voicelines = 0;
    for line in output.lines() {
        if line.starts_with("[REW]") {
            let _round = line[12..13].to_owned().parse::<i32>().unwrap();
            let _voiceline = line[24..25].to_owned().parse::<i32>().unwrap();
            if line.contains("[BEST]") || line.contains("[GOOD]") {
                perfect_voicelines += 1;
            }
        }
    }
    perfect_voicelines
}

pub fn determine_run(lines: String, perfect_runs: i32, attempts: i32, secs: i32, nsecs: i32) {
    kill_libtas();
    let perfect_fruits = perfect_fruits(lines.clone());
    let perfect_patterns = perfect_patterns(lines.clone());
    match (perfect_fruits.0, perfect_patterns.0, perfect_patterns.1.as_str(), perfect_fruits.1.as_str()) {
        (5, 5, _, "banana" | "lemon" | "coconut") => {
            PERFECTS.store(PERFECTS.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
            log(BruteforceLogType::PotentiallyPerfectRun, PERFECTS.load(Ordering::SeqCst), attempts, secs, nsecs, perfect_fruits, perfect_patterns);
        },
        (4, 5, _, "banana" | "lemon" | "coconut") => {
            log(BruteforceLogType::Close, perfect_runs, attempts, secs, nsecs, perfect_fruits, perfect_patterns);
        },
        (5, 4, _, "banana" | "lemon" | "coconut") => {
            log(BruteforceLogType::Close, perfect_runs, attempts, secs, nsecs, perfect_fruits, perfect_patterns);
        },
        (999, _, _, _) => {
            log(BruteforceLogType::Ooh, perfect_runs, attempts, secs, nsecs, perfect_fruits, perfect_patterns);
        }
        _ => {
            log(BruteforceLogType::FailedRun, perfect_runs, attempts, secs, nsecs, perfect_fruits, perfect_patterns);
        }
    }
    ATTEMPTS.store(ATTEMPTS.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
}

pub fn run_libtas(mode: Mode, s: i32, nsec: i32) -> Result<(), Box<dyn Error>> {
    println!("Running libtas at {}s {}ns", s, nsec);
    let ltm_path = match mode {
        Mode::Bruteforcing => consts::BRUTEFORCE_LTM_PATH,
        Mode::Testing => consts::TEST_LTM_PATH,
    };

    // Spawn the child and track its PID so it can be killed precisely if needed.
    let child = Command::new(consts::LIBTAS_PATH)
        .args([
            OsStr::new("--test-mode"),
            OsStr::new("-s"),
            OsStr::new(format!("initial_time_nsec={nsec}").as_str()),
            OsStr::new("-s"),
            OsStr::new(format!("initial_time_sec={s}").as_str()),
            OsStr::new("-r"),
            path::absolute(ltm_path).unwrap().as_os_str(),
            path::absolute(consts::GAME_PATH).unwrap().as_os_str(),
            OsStr::new("-g"),
            OsStr::new("gl"),
            OsStr::new("--no-gui"),
            OsStr::new("/home/luke/faf/ruffle/fetchfruit_tas.swf"),
            OsStr::new("--width"),
            OsStr::new("670"),
            OsStr::new("--height"),
            OsStr::new("422"),
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    // Store the PID in the global tracker so kill_libtas can target it without
    // scanning by process name.
    let pid = child.id();
    {
        let m = LIBTAS_PID.get_or_init(|| Mutex::new(None));
        let mut guard = m.lock().unwrap();
        *guard = Some(pid);
    }

    // Wait for the child and capture output. After completion clear the PID.
    let output = child.wait_with_output()?;
    {
        let m = LIBTAS_PID.get_or_init(|| Mutex::new(None));
        let mut guard = m.lock().unwrap();
        *guard = None;
    }

    let string = String::from_utf8_lossy(&output.stdout).to_string();

    match mode {
        Mode::Bruteforcing => {
            determine_run(string.clone(), PERFECTS.load(Ordering::SeqCst), ATTEMPTS.load(Ordering::SeqCst),
                        CUR_SECS.load(Ordering::SeqCst), CUR_NSECS.load(Ordering::SeqCst));
            CUR_NSECS.store(CUR_NSECS.load(Ordering::SeqCst) + 1000, Ordering::SeqCst);
        },
        Mode::Testing => {
            determine_run_test(string.clone(), TRUE_PERFECTS.load(Ordering::SeqCst), s, nsec);
        },
    }
    Ok(())
}

pub fn kill_libtas() {
    // First, try to kill the tracked PID if present. This avoids scanning by name.
    if let Some(m) = LIBTAS_PID.get() {
        let mut guard = m.lock().unwrap();
        if let Some(pid) = *guard {
            // Try to kill via the system `kill` command. This avoids adding a libc
            // dependency and works on Unix-like systems where this project runs.
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .status();
            *guard = None;
            return;
        }
    }

    // Fallback: scan by process name and kill matches (as before).
    let mut system = sysinfo::System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    for p in system.processes_by_name(OsStr::new("libTAS")) {
        p.kill();
    }
}

pub fn keep_last_n_lines(file_path: &str, n: usize) -> std::io::Result<()> {
    // 1. Read lines backwards up to N lines
    let file = File::open(file_path)?;
    let rev_reader = RevBufReader::new(file);
    
    let mut lines: Vec<String> = rev_reader
        .lines()
        .take(n)
        .collect::<Result<Vec<_>, _>>()?;

    // Since we read backwards, reverse the list to restore correct order
    lines.reverse();

    // 2. Overwrite the file
    let mut file = File::create(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}
