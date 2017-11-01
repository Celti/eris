use chrono::Local;
use log::{self, Log, LogLevel, LogMetadata, LogRecord, SetLoggerError};

struct Logger {
    level: LogLevel,
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "{} {} [{}] {}",
                Local::now().to_string(),
                record.level(),
                record.location().module_path(),
                record.args()
            );
        }
    }
}

pub fn init(level: LogLevel) -> Result<(), SetLoggerError> {
    log::set_logger(|max| {
        max.set(level.to_log_level_filter());
        Box::new(Logger { level: level })
    })
}
