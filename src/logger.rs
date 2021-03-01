use std::io::Write;

pub trait Logger {
    fn info() -> Box<dyn Write>;
    fn err() -> Box<dyn Write>;
}

pub struct StdioLogger;

impl Logger for StdioLogger {
    fn info() -> Box<dyn Write> {
        Box::new(std::io::stdout())
    }

    fn err() -> Box<dyn Write> {
        Box::new(std::io::stderr())
    }
}