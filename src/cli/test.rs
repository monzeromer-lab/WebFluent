//! `wf test [path] [--update]`: the project's tests ([`crate::testing`]),
//! printed as they run.

use std::path::Path;

use crate::error::{Result, WebFluentError};
use crate::testing::{self, Outcome};

pub fn run_test(path: &Path, update: bool) -> Result<()> {
    let tests = testing::collect(path)?;
    if tests.tests.is_empty() {
        println!("no tests: declare one as `test \"name\" {{ … expect \"text\" }}` under tests/");
        return Ok(());
    }
    let mut passed = 0;
    let mut failed = 0;
    let mut written = 0;
    // A test that acts needs a browser, and a browser is expensive to
    // start: one is opened for all of them, and only if any test asks.
    let mut stage = if tests.acting() > 0 {
        match crate::cli::act::Stage::open() {
            Ok(stage) => Some(stage),
            Err(e) => {
                println!(
                    "  {} test(s) act, and there is no browser to run them in",
                    tests.acting()
                );
                println!("  {e}");
                return Err(WebFluentError::IoError(
                    "a test that clicks needs a browser".to_string(),
                ));
            }
        }
    } else {
        None
    };
    for (file, test) in &tests.tests {
        // A test that clicks is run; one that only looks is rendered.
        let outcome = if test.acts() {
            let stage = stage.as_mut().expect("opened above");
            let work = stage.work().to_path_buf();
            tests.run_acting(stage, test, &work)
        } else {
            tests.run_rendered(file, test, update)
        };
        match outcome {
            Outcome::Passed => {
                passed += 1;
                if test.acts() {
                    println!("  ok    {} — {} (in a browser)", file, test.name);
                } else {
                    println!("  ok    {} — {}", file, test.name);
                }
            }
            Outcome::Written => {
                passed += 1;
                written += 1;
                println!("  new   {} — {} (snapshot written)", file, test.name);
            }
            Outcome::Failed(reasons) => {
                failed += 1;
                println!("  FAIL  {} — {}", file, test.name);
                for r in reasons {
                    println!("        {r}");
                }
            }
        }
    }
    println!(
        "{passed} passed, {failed} failed{}",
        if written > 0 {
            format!(", {written} snapshot(s) written")
        } else {
            String::new()
        }
    );
    if failed > 0 {
        return Err(WebFluentError::IoError(format!("{failed} test(s) failed")));
    }
    Ok(())
}
