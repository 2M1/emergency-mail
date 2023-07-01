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
const MAX_LOG_SIZE: u64 = 100 * 1024; // 100KB
#[cfg(debug_assertions)]
const MAX_LOG_SIZE: u64 = 10 * 1024; // 10KB

const MAX_LOG_COUNT: u32 = 10;
#[cfg(not(debug_assertions))]
const FILE_LOG_LEVEL: LevelFilter = LevelFilter::Info;
#[cfg(debug_assertions)]
const FILE_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

pub fn init_logging() -> Handle {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{f}:{L}] - {h({m}{n})}",
        )))
        .build();

    let window_roller = FixedWindowRoller::builder()
        .build("logs/emergency_mails_{}.log", MAX_LOG_COUNT)
        .unwrap();

    let size_trigger = SizeTrigger::new(MAX_LOG_SIZE);
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(window_roller));

    let rolling = RollingFileAppender::builder()
        .append(true)
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{f}:{L}] - {h({m}{n})}",
        )))
        .build("logs/emergency_mails.log", Box::new(policy))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
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

    return log4rs::init_config(config).unwrap();
}
