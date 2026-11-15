//! The guide's links as the app reads them: every page and heading a link
//! names must exist. The check is the guide crate's own, so the app's
//! tests and this task cannot drift apart.

use crate::Failure;

pub fn guide_links(args: &[String]) -> Result<String, Failure> {
    if !args.is_empty() {
        return Err(Failure::Usage);
    }

    let broken = toolbox_guide::broken_links(toolbox_guide::embedded());
    if broken.is_empty() {
        Ok("every link in the guide finds its page and heading".to_owned())
    } else {
        Err(Failure::Problems(broken))
    }
}
