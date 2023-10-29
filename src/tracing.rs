use miette::IntoDiagnostic;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

pub fn install_tracing(filter_directives: &str) -> miette::Result<()> {
    let env_filter = EnvFilter::try_new(filter_directives).into_diagnostic()?;

    let subscriber = tracing_subscriber::fmt::layer()
        .compact()
        .without_time()
        .with_level(false)
        .with_target(false)
        .with_filter(env_filter);

    tracing_subscriber::registry()
        .with(subscriber)
        .try_init()
        .into_diagnostic()
}
