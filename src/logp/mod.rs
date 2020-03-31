use log::{LevelFilter, Record, SetLoggerError};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root, Logger};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::filter::{Filter, Response};

pub fn init() -> Result<(), SetLoggerError> {
    let file_path = "access.log";
    let err_path = "error.log";

    // Build a stderr logger.
    let _stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();
    let errfile = FileAppender::builder()
        //.filter(Box::new(ThresholdFilter::new(log::LevelFilter::Error)))
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(err_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(LogLevelFilter::new(log::LevelFilter::Info)))
                .build("logfile", Box::new(logfile)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Error)))
                .build("errfile", Box::new(errfile)),
        )
        .logger(Logger::builder()
            .appender("logfile")
            .additive(false)
            .build("iperfp::trans", LevelFilter::Info))
        .build(
            Root::builder()
                .appender("errfile")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);

    Ok(())
}

#[derive(Debug)]
pub struct LogLevelFilter {
    level: LevelFilter,
}

impl LogLevelFilter {
    pub fn new(level: LevelFilter) -> LogLevelFilter {
        LogLevelFilter { level }
    }
}

impl Filter for LogLevelFilter {
    fn filter(&self, record: &Record) -> Response {
        if record.level() != self.level {
            Response::Reject
        } else {
            Response::Neutral
        }
    }
}
