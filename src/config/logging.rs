use log::{logger, LevelFilter};
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::{
                compound::{
                    roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger,
                    CompoundPolicy, CompoundPolicyConfig,
                },
                Policy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Config, Logger,
};

const MAX_LOG_SIZE: u64 = 10 * 1024; // 10KB
const MAX_LOG_COUNT: u32 = 10;

pub fn init_logging() {
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
            "{d(%Y-%m-%d %H:%M:%S)} [{f}:{L}] - {m}{n}",
        )))
        .build("logs/emergency_mails.log", Box::new(policy))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Warn)))
                .build("rolling", Box::new(rolling)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("rolling")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}
