use reqwest::StatusCode;

/// Helper function to format HTTP request/response context for logging and
/// error reporting
pub(crate) fn format_http_context<U: AsRef<str>>(
    status: Option<StatusCode>,
    method: &str,
    url: U,
) -> String {
    if let Some(status) = status {
        format!("{} {} {}", status.as_u16(), method, url.as_ref())
    } else {
        format!("{} {}", method, url.as_ref())
    }
}
