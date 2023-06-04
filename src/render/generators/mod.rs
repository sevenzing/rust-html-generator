mod html;
mod html_token;
mod report;

pub use html::HtmlGenerator;
pub use html_token::{HtmlToken, JumpDestination, JumpLocation, Navigation};
pub use report::{MyPath, ReportGenerator};
