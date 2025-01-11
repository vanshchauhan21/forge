use colored::Colorize;

pub enum StatusKind {
    Execute,
    Success,
    Failed,
    Title,
}

impl StatusKind {
    fn icon(&self) -> &'static str {
        match self {
            StatusKind::Execute => "⚙",
            StatusKind::Success => "✓",
            StatusKind::Failed => "✗",
            StatusKind::Title => "◆",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            StatusKind::Execute => "EXECUTE",
            StatusKind::Success => "SUCCESS",
            StatusKind::Failed => "FAILED",
            StatusKind::Title => "TITLE",
        }
    }
}

pub struct StatusDisplay<'a> {
    pub kind: StatusKind,
    pub message: &'a str,
    pub timestamp: Option<String>,
    pub error_details: Option<&'a str>,
}

impl StatusDisplay<'_> {
    pub fn format(&self) -> String {
        let (icon, label, message) = match self.kind {
            StatusKind::Execute => (
                self.icon().cyan(),
                self.label().bold().cyan(),
                format!("{} ...", self.message.bold().cyan()),
            ),
            StatusKind::Success => (
                self.icon().green(),
                self.label().bold().green(),
                self.message.bold().green().to_string(),
            ),
            StatusKind::Failed => {
                let error_suffix = self
                    .error_details
                    .map(|e| format!(" ({})", e))
                    .unwrap_or_default();
                (
                    self.icon().red(),
                    self.label().bold().red(),
                    format!("{}{}", self.message.bold().red(), error_suffix.red()),
                )
            }
            StatusKind::Title => (
                self.icon().blue(),
                self.label().bold().blue(),
                self.message.bold().blue().to_string(),
            ),
        };

        if let Some(timestamp) = &self.timestamp {
            format!(
                "{} {} {} {} {}",
                timestamp.dimmed(),
                icon,
                label,
                "▶".bold(),
                message
            )
        } else {
            format!("{} {} {} {}", icon, label, "▶".bold(), message)
        }
    }

    fn icon(&self) -> &'static str {
        self.kind.icon()
    }

    fn label(&self) -> &'static str {
        self.kind.label()
    }
}
