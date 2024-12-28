use forge_provider::{AnyMessage, Request};

pub struct ContextEngine {
    context: Request,
}

impl ContextEngine {
    pub fn new(context: Request) -> Self {
        Self { context }
    }

    pub fn render_html(&self) -> String {
        // Convert context to markdown format
        let mut markdown = String::new();
        for msg in &self.context.messages {
            let role = match msg {
                AnyMessage::System(_) => "System",
                AnyMessage::User(_) => "User",
                AnyMessage::Assistant(_) => "Assistant",
            };
            markdown.push_str(&format!("## {}\n\n{}\n\n", role, msg.content()));
        }

        // Convert markdown to HTML with basic styling
        let mut html_output = String::from(r#"
            <html>
            <head>
                <style>
                    body { 
                        font-family: system-ui, -apple-system, sans-serif;
                        line-height: 1.5;
                        max-width: 800px;
                        margin: 0 auto;
                        padding: 2rem;
                    }
                    h2 { 
                        color: #2563eb;
                        margin-top: 2rem;
                    }
                    pre {
                        background: #f1f5f9;
                        padding: 1rem;
                        border-radius: 0.5rem;
                        overflow-x: auto;
                    }
                </style>
            </head>
            <body>
        "#);
        
        let parser = pulldown_cmark::Parser::new(&markdown);
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output.push_str("</body></html>");
        
        html_output
    }
}
