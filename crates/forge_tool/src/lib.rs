mod approve;
mod ask;
mod fetch;
mod fs;
mod outline;
mod select;
mod shell;
mod think;

use approve::Approve;
use fetch::Fetch;
use forge_domain::Tool;
use fs::*;
use outline::Outline;
use select::SelectTool;
use shell::Shell;
use think::Think;

pub fn tools() -> Vec<Tool> {
    vec![
        Approve.into(),
        FSRead.into(),
        FSWrite.into(),
        FSList.into(),
        FSSearch.into(),
        FSFileInfo.into(),
        FSReplace.into(),
        Outline.into(),
        SelectTool.into(),
        Shell::default().into(),
        Think::default().into(),
        Fetch::default().into(),
    ]
}
