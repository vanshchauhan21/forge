use std::fmt::Display;

#[derive(Debug)]
pub struct UsagePrompt {
    pub tool_name: String,
    pub input_parameters: Vec<UsageParameterPrompt>,
    pub description: String,
}

impl Display for UsagePrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tool_name)?;
        f.write_str("\n")?;
        f.write_str(&self.description)?;

        f.write_str("\n\nUsage:\n")?;
        f.write_str("<tool_call>\n")?;
        f.write_str("<")?;
        f.write_str(&self.tool_name)?;
        f.write_str(">")?;

        for parameter in self.input_parameters.iter() {
            f.write_str("\n")?;
            parameter.fmt(f)?;
        }

        f.write_str("\n")?;
        f.write_str("</")?;
        f.write_str(&self.tool_name)?;
        f.write_str(">\n")?;
        f.write_str("</tool_call>\n")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct UsageParameterPrompt {
    pub parameter_name: String,
    pub parameter_type: String,
}

impl Display for UsageParameterPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<")?;
        f.write_str(&self.parameter_name)?;
        f.write_str(">")?;
        f.write_str(&self.parameter_type)?;
        f.write_str("</")?;
        f.write_str(&self.parameter_name)?;
        f.write_str(">")?;

        Ok(())
    }
}
