use std::io::Write;

#[allow(dead_code)] // TODO: Remove
#[derive(Default, Eq, PartialEq)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

#[allow(dead_code)]
pub struct Logger {
    out: Box<dyn Write>,
}

#[allow(dead_code)]
impl Logger {
    fn log(self: &mut Logger, level: LogLevel, message: impl Into<String>) {
        match level {
            LogLevel::Debug => {
                let _ = writeln!(self.out, "DEBUG: {}", message.into());
            }
            LogLevel::Info => {
                let _ = writeln!(self.out, "INFO: {}", message.into());
            }
            LogLevel::Warning => {
                let _ = writeln!(self.out, "WARNING: {}", message.into());
            }
            LogLevel::Error => {
                let _ = writeln!(self.out, "ERROR: {}", message.into());
            }
        }
    }

    pub fn debug(self: &mut Logger, message: impl Into<String>) {
        self.log(LogLevel::Debug, message)
    }

    pub fn info(self: &mut Logger, message: impl Into<String>) {
        self.log(LogLevel::Info, message)
    }

    pub fn warning(self: &mut Logger, message: impl Into<String>) {
        self.log(LogLevel::Warning, message)
    }

    pub fn error(self: &mut Logger, message: impl Into<String>) {
        self.log(LogLevel::Error, message)
    }
}

#[allow(dead_code)] // TODO: Remove
pub fn new_logger() -> Logger {
    let output: Box<dyn Write> = Box::new(std::io::stdout());
    Logger { out: output }
}
