use colored::Colorize;

#[derive(Clone)]
enum Kind {
    Execute,
    Success,
    Failed,
    Title,
}

impl Kind {
    fn icon(&self) -> &'static str {
        match self {
            Kind::Execute => "⚙",
            Kind::Success => "✓",
            Kind::Failed => "✗",
            Kind::Title => "◆",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Kind::Execute => "EXECUTE",
            Kind::Success => "SUCCESS",
            Kind::Failed => "FAILED",
            Kind::Title => "TITLE",
        }
    }
}

#[derive(Clone)]
pub struct StatusDisplay {
    kind: Kind,
    message: String,
    error_details: Option<String>,
}

impl StatusDisplay {
    /// Create a status for executing a tool
    pub fn execute(message: impl Into<String>) -> Self {
        Self {
            kind: Kind::Execute,
            message: message.into(),
            error_details: None,
        }
    }

    /// Create a success status
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            kind: Kind::Success,
            message: message.into(),
            error_details: None,
        }
    }

    /// Create a failure status
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            kind: Kind::Failed,
            message: message.into(),
            error_details: None,
        }
    }

    /// Create a failure status with additional details
    pub fn failed_with(message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            kind: Kind::Failed,
            message: message.into(),
            error_details: Some(details.into()),
        }
    }

    /// Create a title status
    pub fn title(message: impl Into<String>) -> Self {
        Self {
            kind: Kind::Title,
            message: message.into(),
            error_details: None,
        }
    }

    pub fn format(&self) -> String {
        let (icon, label, message) = match self.kind {
            Kind::Execute => (
                self.icon().cyan(),
                self.label().bold().cyan(),
                format!("{} ...", self.message.bold().cyan()),
            ),
            Kind::Success => (
                self.icon().green(),
                self.label().bold().green(),
                self.message.bold().green().to_string(),
            ),
            Kind::Failed => {
                let error_suffix = self
                    .error_details
                    .as_ref()
                    .map(|e| format!(" ({})", e))
                    .unwrap_or_default();
                (
                    self.icon().red(),
                    self.label().bold().red(),
                    format!("{}{}", self.message.bold().red(), error_suffix.red()),
                )
            }
            Kind::Title => (
                self.icon().blue(),
                self.label().bold().blue(),
                self.message.bold().blue().to_string(),
            ),
        };

        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        format!(
            "{} {} {} {} {}",
            timestamp.dimmed(),
            icon,
            label,
            "▶".bold(),
            message
        )
    }

    fn icon(&self) -> &'static str {
        self.kind.icon()
    }

    fn label(&self) -> &'static str {
        self.kind.label()
    }
}
