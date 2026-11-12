//! Chrome, for a test that acts: the browser `wf test` plays one in
//! ([`crate::testing::Browser`]). The site is served from a port of this
//! machine and driven over the DevTools protocol.

use std::path::{Path, PathBuf};

use crate::browser::Browser;
use crate::browser::chrome::Page;
use crate::error::Result;

/// A browser, kept for as long as there are tests that need one.
pub struct Stage {
    browser: Browser,
    work: PathBuf,
}

impl Stage {
    pub fn open() -> Result<Self> {
        let browser = Browser::start()?;
        // A directory of this run's own, which nobody else could have made
        // first, and which goes with the stage.
        let work = crate::browser::chrome::private_dir("wf-act-").map_err(|e| {
            crate::error::WebFluentError::IoError(format!(
                "could not make a directory for the tests' sites: {e}"
            ))
        })?;
        Ok(Stage { browser, work })
    }

    /// Where the tests' sites are built.
    pub fn work(&self) -> &Path {
        &self.work
    }
}

impl crate::testing::Browser for Stage {
    fn open(&mut self, dir: &Path, settle_ms: u64) -> Result<Box<dyn crate::testing::Page + '_>> {
        let server = super::preview::serve_directory(dir.to_path_buf(), "")?;
        let page = self
            .browser
            .open(&format!("{}/", server.origin), settle_ms)?;
        Ok(Box::new(ChromePage {
            browser: &mut self.browser,
            page: Some(page),
            server: Some(server),
        }))
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.work);
    }
}

/// A test's page in Chrome, and the server it came from; both close when
/// it is dropped.
struct ChromePage<'a> {
    browser: &'a mut Browser,
    page: Option<Page>,
    server: Option<super::preview::Preview>,
}

impl crate::testing::Page for ChromePage<'_> {
    fn eval(&mut self, script: &str) -> Result<serde_json::Value> {
        let page = self.page.as_ref().expect("open until dropped");
        self.browser.eval(page, script)
    }

    fn settle(&mut self, ms: u64) -> Result<()> {
        let page = self.page.as_mut().expect("open until dropped");
        self.browser.settle(page, ms)
    }

    fn errors(&self) -> Vec<String> {
        self.page
            .as_ref()
            .map(|p| p.errors().to_vec())
            .unwrap_or_default()
    }
}

impl Drop for ChromePage<'_> {
    fn drop(&mut self) {
        if let Some(page) = self.page.take() {
            let _ = self.browser.close(page);
        }
        if let Some(server) = self.server.take() {
            server.close();
        }
    }
}
