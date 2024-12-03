use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::ParseError;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() -> Result<(), ParseError>{
    let env_filter =
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .parse("")?;
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .init();
    Ok(())
}