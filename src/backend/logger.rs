/*
  Copyright 2024 Mike Nguyen (mikeee) <hey@mike.ee>

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/
use std::io::Write;

#[allow(dead_code)] // TODO: Remove
#[derive(Default, Eq, PartialEq, Debug)]
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    struct TestWrite {
        buffer: Rc<RefCell<Vec<u8>>>,
    }

    impl Write for TestWrite {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn create_test_logger() -> (Logger, Rc<RefCell<Vec<u8>>>) {
        let buffer = Rc::new(RefCell::new(Vec::new()));
        let test_write = TestWrite {
            buffer: Rc::clone(&buffer),
        };
        let logger = Logger {
            out: Box::new(test_write),
        };
        (logger, buffer)
    }

    fn get_output(buffer: &Rc<RefCell<Vec<u8>>>) -> String {
        String::from_utf8(buffer.borrow().clone()).unwrap()
    }

    #[test]
    fn test_log_levels() {
        let (mut logger, buffer) = create_test_logger();

        logger.debug("Debug message");
        logger.info("Info message");
        logger.warning("Warning message");
        logger.error("Error message");

        let output = get_output(&buffer);
        assert_eq!(output, "DEBUG: Debug message\nINFO: Info message\nWARNING: Warning message\nERROR: Error message\n");
    }

    #[test]
    fn test_log_method() {
        let (mut logger, buffer) = create_test_logger();

        logger.log(LogLevel::Debug, "Test debug");
        logger.log(LogLevel::Info, "Test info");
        logger.log(LogLevel::Warning, "Test warning");
        logger.log(LogLevel::Error, "Test error");

        let output = get_output(&buffer);
        assert_eq!(
            output,
            "DEBUG: Test debug\nINFO: Test info\nWARNING: Test warning\nERROR: Test error\n"
        );
    }

    #[test]
    fn test_new_logger() {
        let mut logger = new_logger();
        logger.info("Test message");
        // If this doesn't panic, we can assume it's working correctly
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn test_into_string() {
        let (mut logger, buffer) = create_test_logger();

        logger.info(String::from("String message"));
        logger.info("&str message");

        let output = get_output(&buffer);
        assert_eq!(output, "INFO: String message\nINFO: &str message\n");
    }
}
