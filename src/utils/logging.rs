use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use crate::config::LogLevel;

/// Initialize logging based on the configuration
pub fn init_logging(log_level: &LogLevel) {
    let level = match log_level {
        LogLevel::Error => Level::ERROR,
        LogLevel::Warn => Level::WARN,
        LogLevel::Info => Level::INFO, 
        LogLevel::Debug => Level::DEBUG,
        LogLevel::Off => return, // Don't initialize logging if disabled
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Log a request being sent
pub fn log_request(method: &str, url: &str, body: Option<&str>) {
    tracing::debug!(
        method = method,
        url = url,
        body = body,
        "Sending request"
    );
}

/// Log a response received
pub fn log_response(status: u16, body: Option<&str>, request_id: Option<&str>) {
    tracing::debug!(
        status = status,
        body = body,
        request_id = request_id,
        "Received response"
    );
} 