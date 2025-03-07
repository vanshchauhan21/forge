use std::collections::HashMap;
use std::time::Duration;

use thiserror::Error;
use tiny_http::{Header, Method, Response, Server, StatusCode};
use url::form_urlencoded;

/// Errors that can occur during callback handling
#[derive(Error, Debug)]
pub enum CallbackError {
    #[error("Callback timed out after {0} seconds")]
    Timeout(u64),

    #[error("Server error: {0}")]
    ServerError(String),
}

/// The result of a successful callback
#[derive(Debug, Clone)]
pub struct CallbackResult {
    pub state: String,
    pub code: String,
}

/// A server handle structure that allows us to explicitly shut down the server
/// from outside
pub struct ServerHandle {
    server_task: tokio::task::JoinHandle<()>,
    shutdown_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ServerHandle {
    /// Explicitly shut down the server
    pub async fn shutdown(&self) {
        // Signal the server to shut down
        self.shutdown_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Give it a short time to exit cleanly
        tokio::time::sleep(Duration::from_millis(100)).await;

        // If it's still running, abort it
        self.server_task.abort();
    }
}
pub struct CallbackServer {
    port: u16,
}

impl Default for CallbackServer {
    fn default() -> Self {
        Self { port: 8080 }
    }
}

// HTML templates included from files with standardized naming
const SUCCESS_PAGE_HTML: &str = include_str!("html_templates/success_page.html");
const ERROR_PAGE_HTML: &str = include_str!("html_templates/error_page.html");
const MISSING_PARAMS_PAGE_HTML: &str = include_str!("html_templates/missing_params_page.html");
const NOT_FOUND_PAGE_HTML: &str = include_str!("html_templates/not_found_page.html");

// HTML template functions
fn success_html() -> String {
    SUCCESS_PAGE_HTML.to_string()
}

fn error_html(error_message: &str) -> String {
    ERROR_PAGE_HTML.replace("{}", error_message)
}

fn missing_params_html() -> String {
    MISSING_PARAMS_PAGE_HTML.to_string()
}

fn not_found_html() -> String {
    NOT_FOUND_PAGE_HTML.to_string()
}

impl CallbackServer {
    /// Start the callback server and return a handle to it
    pub async fn start_server(
        &self,
    ) -> Result<(ServerHandle, tokio::sync::oneshot::Receiver<CallbackResult>), CallbackError> {
        // Create a channel to send the callback result
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Create a tiny_http server with retries for port binding
        let server_result = Server::http(format!("127.0.0.1:{}", self.port));

        // If we couldn't bind to the port, it might be because a previous server is
        // still shutting down
        let server = match server_result {
            Ok(server) => server,
            Err(e) => {
                // Wait a short time and try again - this helps in the login-logout-login
                // scenario
                eprintln!(
                    "Failed to start server: {}. Retrying after a short delay...",
                    e
                );
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Try again
                Server::http(format!("127.0.0.1:{}", self.port)).map_err(|e| {
                    CallbackError::ServerError(format!("Failed to start server after retry: {}", e))
                })?
            }
        };

        // Clone the sender to move into the closure
        let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

        // Create a flag to signal server shutdown
        let shutdown_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_flag_clone = shutdown_flag.clone();

        // Spawn a task to handle incoming requests
        let server_task = tokio::task::spawn_blocking(move || {
            // Process requests with a timeout to make it effectively non-blocking
            let timeout = Duration::from_millis(100);

            loop {
                // Check if we should shut down
                if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // Try to receive a request with a timeout
                match server.recv_timeout(timeout) {
                    Ok(Some(request)) => {
                        // Only handle GET requests to /callback
                        if request.method() == &Method::Get
                            && request.url().starts_with("/callback")
                        {
                            // Parse the query parameters
                            let query = request.url().split('?').nth(1).unwrap_or("");
                            let params: HashMap<String, String> =
                                form_urlencoded::parse(query.as_bytes())
                                    .into_owned()
                                    .collect();

                            let content_type =
                                Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                                    .expect("Valid header");

                            let response = if let Some(err) = params.get("error") {
                                // Handle error case
                                Response::from_string(error_html(err))
                                    .with_header(content_type)
                                    .with_status_code(StatusCode(400))
                            } else if let (Some(state), Some(code)) =
                                (params.get("state"), params.get("code"))
                            {
                                // Handle successful case
                                let result = CallbackResult {
                                    state: state.to_string(),
                                    code: code.to_string(),
                                };

                                // Send the result through the channel
                                if let Some(tx) = tx.lock().unwrap().take() {
                                    let _ = tx.send(result);
                                }

                                Response::from_string(success_html())
                                    .with_header(content_type)
                                    .with_status_code(StatusCode(200))
                            } else {
                                // Handle missing parameters case
                                Response::from_string(missing_params_html())
                                    .with_header(content_type)
                                    .with_status_code(StatusCode(400))
                            };

                            // Send the response
                            if let Err(e) = request.respond(response) {
                                eprintln!("Failed to send response: {}", e);
                            }
                        } else {
                            // For any other request, return a not found response
                            let content_type =
                                Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                                    .expect("Valid header");

                            let response = Response::from_string(not_found_html())
                                .with_header(content_type)
                                .with_status_code(StatusCode(404));

                            if let Err(e) = request.respond(response) {
                                eprintln!("Failed to send response: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        // No request received, continue waiting
                    }
                    Err(e) => {
                        eprintln!("Error receiving request: {}", e);
                    }
                }
            }
            // Explicitly drop the server to close the socket
            drop(server);
        });

        let server_handle = ServerHandle { server_task, shutdown_flag: shutdown_flag_clone };

        Ok((server_handle, rx))
    }

    /// Wait for a callback for the specified duration
    pub async fn wait_for_callback_with_handle(
        &self,
        timeout_secs: u64,
    ) -> Result<(CallbackResult, ServerHandle), CallbackError> {
        // Start the server
        let (server_handle, rx) = self.start_server().await?;

        // Wait for the callback with timeout
        match tokio::time::timeout(Duration::from_secs(timeout_secs), rx).await {
            Ok(Ok(result)) => Ok((result, server_handle)),
            Ok(Err(_)) => {
                // Channel error (shouldn't happen)
                server_handle.shutdown().await;
                Err(CallbackError::ServerError(
                    "Failed to receive callback".to_string(),
                ))
            }
            Err(_) => {
                // Timeout
                server_handle.shutdown().await;
                Err(CallbackError::Timeout(timeout_secs))
            }
        }
    }
}
