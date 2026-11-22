use std::path::Path;

use crate::agent::context::{Depth, context_via, guide};
use crate::diagnostics::error::{Result, WebFluentError};

/// `wf context [-d DIR] [--open FILE[:START-END]] [--guide index|full]
/// [--json]`: what an agent is given — the files, the open file and its
/// selection, the problems, the components, the theme's tokens, the
/// config with only its public `env`, the language reference (the skill's
/// `references/`), and what was withheld.
pub(crate) fn run_context(
    dir: &Path,
    open: Option<&str>,
    guide_depth: &str,
    json: bool,
) -> Result<()> {
    let depth = match guide_depth {
        "index" => Depth::Index,
        "full" => Depth::Full,
        other => {
            return Err(WebFluentError::edit(format!(
                "`--guide {other}`: the reference is given as `index` (each file's sections) or `full` (whole)"
            )));
        }
    };
    let open = open.map(|o| match o.rsplit_once(':') {
        Some((file, range)) => match range.split_once('-') {
            Some((a, b)) => match (a.parse::<usize>(), b.parse::<usize>()) {
                (Ok(a), Ok(b)) => (file, Some((a, b))),
                _ => (o, None),
            },
            None => (o, None),
        },
        None => (o, None),
    });
    let context = context_via(&crate::vfs::FsVfs, dir, open, guide(), depth)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&context).unwrap_or_default()
        );
    } else {
        print!("{}", context.to_markdown());
    }
    Ok(())
}
