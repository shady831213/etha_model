pub const STATICS_LEVEL: tracing::Level = tracing::Level::INFO;

#[cfg(not(test))]
mod ffi {
    pub const STATICS_REG_LEVEL: tracing::Level = tracing::Level::DEBUG;

    use tracing::Subscriber;
    use tracing_chrome::{ChromeLayer, ChromeLayerBuilder};
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter, fmt, registry, EnvFilter};

    pub fn statics<S>(file: &'static str) -> (ChromeLayer<S>, tracing_chrome::FlushGuard)
    where
        S: Subscriber + for<'span> registry::LookupSpan<'span> + Send + Sync,
    {
        ChromeLayerBuilder::new()
            .include_args(true)
            .file(&format!("{}.trace.json", file))
            .include_locations(false)
            .build()
    }
    pub fn default<S>() -> filter::Filtered<
        fmt::Layer<
            S,
            fmt::format::Pretty,
            fmt::format::Format<fmt::format::Pretty>,
            tracing_appender::non_blocking::NonBlocking,
        >,
        EnvFilter,
        S,
    >
    where
        S: Subscriber + for<'span> registry::LookupSpan<'span> + Send + Sync,
    {
        let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
        std::mem::forget(_guard);
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(""))
            .unwrap();
        let fmt_layer = fmt::layer()
            .pretty()
            .with_writer(non_blocking)
            .with_filter(filter_layer);
        fmt_layer
    }
}
#[cfg(not(test))]
pub use ffi::*;
