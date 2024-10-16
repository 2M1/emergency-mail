use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Config, Handle,
};

#[cfg(not(debug_assertions))]
const MAX_LOG_SIZE: u64 = 500 * 1024;
// 500KB
#[cfg(debug_assertions)]
const MAX_LOG_SIZE: u64 = 10 * 1024; // 10KB

const MAX_LOG_COUNT: u32 = 10;
#[cfg(not(debug_assertions))]
const FILE_LOG_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(debug_assertions)]
const FILE_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

/// The pattern used for logging.
/// @see [https://docs.rs/log4rs/1.2.0/log4rs/encode/pattern/index.html](https://docs.rs/log4rs/1.2.0/log4rs/encode/pattern/index.html)
const LOG_PATTERN: &'static str = "{d(%Y-%m-%d %H:%M:%S)} {l}: [{f}:{L}] - {h({m}{n})}";

pub fn init_logging() -> Result<Handle, String> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .build();

    let window_roller = FixedWindowRoller::builder()
        .build("logs/emergency_mails_{}.log", MAX_LOG_COUNT)
        .map_err(|e| e.to_string())?;

    let size_trigger = SizeTrigger::new(MAX_LOG_SIZE);
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(window_roller));

    let rolling = RollingFileAppender::builder()
        .append(true)
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .build("logs/emergency_mails.log", Box::new(policy))
        .unwrap();

    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Trace)))
                .build("stdout", Box::new(stdout)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(FILE_LOG_LEVEL)))
                .build("rolling", Box::new(rolling)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("rolling")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    return Ok(log4rs::init_config(config).map_err(|e| e.to_string())?);
}
