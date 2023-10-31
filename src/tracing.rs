use miette::IntoDiagnostic;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

type ReloadHandle = Handle<EnvFilter, Registry>;

/// The default filter directive.
pub const DEFAULT_FILTER: &str = "info";

pub fn install_tracing(
    filter_directives: &str,
) -> std::result::Result<ReloadHandle, miette::Report> {
    let env_filter = EnvFilter::try_new(filter_directives).into_diagnostic()?;

    let (env_filter, reload_handle) = tracing_subscriber::reload::Layer::new(env_filter);

    let subscriber = tracing_subscriber::fmt::layer()
        .compact()
        .without_time()
        .with_level(false)
        .with_target(false)
        .with_filter(env_filter);

    tracing_subscriber::registry()
        .with(subscriber)
        .try_init()
        .into_diagnostic()?;

    Ok(reload_handle)
}

pub fn update_log_filters(handle: &ReloadHandle, filter_directives: &str) -> miette::Result<()> {
    let env_filter = EnvFilter::try_new(filter_directives).into_diagnostic()?;

    handle
        .modify(|old_filter| {
            *old_filter = env_filter;
        })
        .into_diagnostic()
}
