//! Text in any script, as the docs site writes it: Arabic inside a
//! string's splices, beside the name a request looks for. Finding a name's
//! uses reads every string of the project, so one request reads them all —
//! and a byte inside a letter is never where it looks.

use lsp_types::*;
use webfluent::syntax::lexer::TokenType;
use wf_lsp::analysis::tokens_of;
use wf_lsp::project::{FileCache, Project};

fn position_of(project: &Project, ix: usize, offset: usize) -> Position {
    let file = &project.files[ix];
    file.index.offset_to_position(&file.source, offset)
}

#[test]
fn a_name_beside_arabic_in_a_splice_is_found_and_nothing_breaks() {
    let src = "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    state name = \"x\"\n    Heading(\"Home\").h1\n    Text(\"أهلاً {name}، مرحبا\")\n    Text(\"{name}أ\")\n    Text(\"{أname}\")\n}\n";
    let project = Project::single(Url::parse("file:///home.wf").expect("a URL"), src);
    let at = src.find("name =").expect("the state");
    let found = wf_lsp::rename::references(&project, 0, position_of(&project, 0, at), true);
    // The declaration and the first two splices; in the third an Arabic
    // letter is part of the name, so it is another name.
    assert_eq!(found.len(), 3, "{found:?}");
}

#[test]
fn the_docs_site_s_arabic_files_answer_every_request_that_reads_strings() {
    let site = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site");
    let Ok(home) = site.join("src/pages/Home.wf").canonicalize() else {
        return;
    };
    let project = Project::load_with(
        &Url::from_file_path(&home).expect("a path"),
        &webfluent::vfs::FsVfs,
        &FileCache::default(),
    );
    let arabic: Vec<usize> = project
        .files
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.source
                .chars()
                .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
        })
        .filter(|(_, f)| !f.script && !f.markdown)
        .map(|(ix, _)| ix)
        .collect();
    assert!(!arabic.is_empty(), "the site has pages in Arabic");
    for ix in arabic {
        let Some(tokens) = tokens_of(&project.files[ix]) else {
            continue;
        };
        // A name in each, and what a request reads for it: every string of
        // the project.
        for token in tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Identifier(_)))
            .take(2)
        {
            let at = position_of(&project, ix, token.offset);
            let _ = wf_lsp::rename::references(&project, ix, at, true);
            let _ = wf_lsp::rename::highlights(&project, ix, at);
            let _ = wf_lsp::rename::prepare_rename(&project, ix, at);
        }
    }
}
