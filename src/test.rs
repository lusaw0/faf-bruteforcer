use crate::{helpers::{kill_libtas, perfect_fruits, perfect_patterns, perfect_voicelines}, log::{TestingLogType, format_log_timestamp, write_and_flush}, consts::TESTING_LOG_FORMATTED};

pub fn determine_run_test(lines: String, true_perfect_runs: i32, secs: i32, nsecs: i32) {
    kill_libtas();
    let perfect_voicelines = perfect_voicelines(lines.clone());
    let perfect_fruits = perfect_fruits(lines.clone());
    let perfect_patterns = perfect_patterns(lines.clone());
    match (perfect_voicelines, perfect_fruits.0, perfect_patterns.0, perfect_patterns.1.as_str(), perfect_fruits.1.as_str()) {
        (4, 5, 5, _, "banana" | "lemon" | "coconut") => {

        },
        _ => {
            log_test(TestingLogType::FailedRun, true_perfect_runs, secs, nsecs);
        }
    }
}

pub fn log_test(log_type: TestingLogType, true_perfect_runs: i32, secs: i32, nsecs: i32) {
    let msg = format_messages_test(log_type,
        true_perfect_runs,
        secs,
        nsecs,
    );
    write_and_flush(TESTING_LOG_FORMATTED, &msg);
}

fn format_messages_test(log_type: TestingLogType,
   true_perfect_runs: i32,
   secs: i32,
   nsecs: i32,
) -> String {
    let heading = match log_type {
        TestingLogType::FailedRun => "Failed! ",
        TestingLogType::TruePerfectRun => "PERFECT!",
    };
    let formatted = format!("{:04} | {:03?}s {:09?}ns | {} | {}",
        true_perfect_runs,
        secs,
        nsecs,
        format_log_timestamp(),
        heading,
    );
    formatted
}
