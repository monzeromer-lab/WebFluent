//! `wf inspect` — one page, as a reader meets it, in a few lines of text.
//!
//! A page WebFluent builds is drawn by its script, so reading the HTML a
//! build wrote says little of what a reader sees. This opens the page in a
//! headless Chrome, lets it load what it asks for, and prints what is there:
//! its landmarks, headings, links and where they go, buttons and their
//! states, fields and their values, tables, lists, pictures and text —
//! with anything that threw, logged an error or failed to arrive. An agent
//! checks a page with one command rather than a browser script of its own,
//! and reads a few hundred tokens rather than a screenshot.
//!
//! A route (`/pokemon/pikachu`) is the project's build, served as
//! `wf start` serves it, `--proxy` included; a URL is a server already
//! running.

use std::path::{Path, PathBuf};

use crate::browser::Browser;
use crate::diagnostics::error::{Result, WebFluentError};

/// The outline the page writes of itself: `{ title, url, lang, lines }`.
/// `__MAX__` is the most lines it writes.
const OUTLINE: &str = r#"(() => {
  const MAX = __MAX__;
  const out = [];
  const clean = (s) => (s || '').replace(/\s+/g, ' ').trim();
  const cut = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
  const pad = (d) => '  '.repeat(d);
  const hidden = (el) => {
    if (el.closest('[hidden],[inert],[aria-hidden="true"]')) return true;
    if (el.tagName === 'DIALOG' && !el.open) return true;
    const cs = getComputedStyle(el);
    return cs.display === 'none' || cs.visibility === 'hidden';
  };
  const name = (el) => {
    const by = el.getAttribute('aria-labelledby');
    if (by) return clean(by.split(/\s+/).map((i) => document.getElementById(i)?.textContent || '').join(' '));
    const label = el.getAttribute('aria-label');
    if (label) return clean(label);
    if (el.labels && el.labels.length) return clean([...el.labels].map((l) => l.textContent).join(' '));
    if (el.tagName === 'IMG') return clean(el.getAttribute('alt'));
    const text = clean(el.innerText || el.textContent);
    return text || clean(el.getAttribute('title') || el.getAttribute('placeholder') || '');
  };
  const states = (el) => {
    const s = [];
    if (el.disabled || el.getAttribute('aria-disabled') === 'true') s.push('disabled');
    for (const k of ['pressed', 'expanded', 'selected', 'checked', 'current', 'invalid', 'busy']) {
      const v = el.getAttribute('aria-' + k);
      if (v && v !== 'false') s.push(v === 'true' ? k : k + '=' + v);
    }
    if ((el.type === 'checkbox' || el.type === 'radio') && el.checked && !s.includes('checked')) s.push('checked');
    return s.length ? ' [' + s.join(', ') + ']' : '';
  };
  const LANDMARK = { HEADER: 'banner', NAV: 'navigation', MAIN: 'main', ASIDE: 'complementary', FOOTER: 'contentinfo', FORM: 'form', DIALOG: 'dialog' };
  const ROLES = ['banner', 'navigation', 'main', 'complementary', 'contentinfo', 'search', 'region', 'dialog', 'alertdialog', 'alert', 'status', 'form', 'tablist', 'tabpanel', 'toolbar', 'menu', 'listbox', 'tree', 'grid'];
  const CONTROLS = ['button', 'tab', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'option', 'switch', 'checkbox', 'radio', 'link', 'treeitem', 'slider', 'combobox'];
  const INLINE = ['SPAN', 'B', 'I', 'EM', 'STRONG', 'CODE', 'SMALL', 'MARK', 'ABBR', 'TIME', 'SUP', 'SUB', 'KBD', 'S', 'U', 'Q', 'CITE', 'DFN', 'VAR', 'SAMP', 'BDI', 'BDO', 'WBR', 'BR'];
  let text = '';
  const flush = (d) => {
    const t = clean(text);
    text = '';
    if (t && out.length < MAX) out.push(pad(d) + '"' + cut(t, 240) + '"');
  };
  const line = (d, s) => {
    flush(d);
    if (out.length < MAX) out.push(pad(d) + s);
  };
  const walk = (node, d) => {
    if (out.length >= MAX) return;
    if (node.nodeType === 3) { text += ' ' + node.textContent; return; }
    if (node.nodeType !== 1) return;
    const el = node;
    const tag = el.tagName.toUpperCase();
    if (['SCRIPT', 'STYLE', 'TEMPLATE', 'NOSCRIPT', 'SVG', 'LINK', 'META'].includes(tag)) return;
    if (hidden(el)) return;
    const role = el.getAttribute('role');
    if (/^H[1-6]$/.test(tag) || role === 'heading') {
      const level = /^H[1-6]$/.test(tag) ? tag.toLowerCase() : 'h' + (el.getAttribute('aria-level') || '2');
      line(d, level + ' ' + cut(name(el), 160));
      return;
    }
    if (tag === 'A' && el.hasAttribute('href')) {
      line(d, 'link "' + cut(name(el), 100) + '" → ' + el.getAttribute('href') + states(el));
      return;
    }
    if (tag === 'BUTTON' || (role && CONTROLS.includes(role))) {
      line(d, (role || 'button') + ' "' + cut(name(el), 100) + '"' + states(el));
      return;
    }
    if (tag === 'INPUT') {
      const t = (el.type || 'text').toLowerCase();
      if (t === 'hidden') return;
      if (t === 'checkbox' || t === 'radio') { line(d, t + ' "' + cut(name(el), 100) + '"' + states(el)); return; }
      if (t === 'submit' || t === 'button') { line(d, 'button "' + cut(el.value || name(el), 100) + '"' + states(el)); return; }
      line(d, (t === 'text' ? 'textbox' : t) + ' "' + cut(name(el), 100) + '" = "' + cut(el.value || '', 80) + '"' + states(el));
      return;
    }
    if (tag === 'TEXTAREA') { line(d, 'textarea "' + cut(name(el), 100) + '" = "' + cut(el.value || '', 80) + '"' + states(el)); return; }
    if (tag === 'SELECT') {
      const o = el.options[el.selectedIndex];
      line(d, 'select "' + cut(name(el), 100) + '" = "' + clean(o && o.text) + '" (' + el.options.length + ' options)' + states(el));
      return;
    }
    if (tag === 'IMG') {
      const alt = el.getAttribute('alt');
      line(d, 'img ' + (alt === null ? '(no alt)' : '"' + cut(clean(alt), 100) + '"') + (el.complete && el.naturalWidth === 0 ? ' [broken]' : ''));
      return;
    }
    if (tag === 'TABLE') {
      const rows = [...el.rows].filter((r) => !hidden(r));
      const caption = clean(el.caption && el.caption.textContent);
      line(d, 'table' + (caption ? ' "' + cut(caption, 100) + '"' : '') + ' (' + rows.length + ' rows)');
      for (const r of rows.slice(0, 6)) {
        if (out.length >= MAX) break;
        out.push(pad(d + 1) + '| ' + [...r.cells].map((c) => cut(clean(c.innerText), 40)).join(' | ') + ' |');
      }
      if (rows.length > 6) out.push(pad(d + 1) + '… ' + (rows.length - 6) + ' more rows');
      return;
    }
    if (tag === 'UL' || tag === 'OL' || role === 'list') {
      const items = [...el.children].filter((c) => !hidden(c));
      line(d, 'list (' + items.length + ' items)');
      for (const item of items.slice(0, 8)) {
        const plain = !item.querySelector('a[href],button,input,select,textarea,[role=button],ul,ol,table,h1,h2,h3,h4,h5,h6,img');
        if (plain) { const t = clean(item.innerText); if (t && out.length < MAX) out.push(pad(d + 1) + '- ' + cut(t, 160)); }
        else walk(item, d + 1);
        flush(d + 1);
      }
      if (items.length > 8) out.push(pad(d + 1) + '… ' + (items.length - 8) + ' more items');
      return;
    }
    if (tag === 'LABEL') {
      // A label names its control, which says it; only its controls are read.
      for (const c of el.children) walk(c, d);
      return;
    }
    let inner = d;
    const landmark = role && ROLES.includes(role) ? role : LANDMARK[tag];
    if (landmark || tag === 'LI') {
      const label = el.getAttribute('aria-label') || (el.getAttribute('aria-labelledby') ? name(el) : '');
      line(d, (tag === 'LI' ? 'item' : landmark) + (label && tag !== 'LI' ? ' "' + cut(clean(label), 80) + '"' : ''));
      inner = d + 1;
    }
    const block = !INLINE.includes(tag);
    if (block) flush(d);
    for (const child of el.childNodes) walk(child, inner);
    if (block) flush(inner);
  };
  walk(document.body, 0);
  flush(0);
  if (out.length >= MAX) out.push('… (more: --lines raises the limit)');
  return JSON.stringify({ title: document.title, url: location.href, lang: document.documentElement.lang, dir: document.documentElement.dir, lines: out });
})()"#;

/// How `wf inspect` looks at a page.
pub(crate) struct Look<'a> {
    pub width: u32,
    pub height: u32,
    /// Laid out as a phone.
    pub mobile: bool,
    /// Where to save a PNG, if anywhere.
    pub screenshot: Option<&'a Path>,
    /// The whole page in the screenshot, not only the window.
    pub full_page: bool,
    /// The most milliseconds the page is given to settle.
    pub wait_ms: u64,
    /// The most lines of outline.
    pub lines: usize,
    /// Steps played on the page before it is read, as a test writes them:
    /// `click "Name"`, `type "pika" into "Search"`, `press "Enter"`.
    pub steps: &'a [String],
    /// `--fakes FILE`: the site built with that file's fakes answering its
    /// endpoints, as `wf verify --fakes` builds it, rather than the build.
    pub fakes: Option<&'a Path>,
}

pub(crate) fn run_inspect(
    target: &str,
    project_dir: &Path,
    proxies: &[String],
    options: &Look<'_>,
    report: super::Report,
    allow_output_outside: bool,
) -> Result<()> {
    let Look {
        width,
        height,
        mobile,
        screenshot,
        ..
    } = *options;
    let json = report == super::Report::Json;
    let proxies = proxies
        .iter()
        .map(|p| super::start::Proxy::parse(p))
        .collect::<Result<Vec<_>>>()?;
    // A URL is a server already running; a route is this project's build.
    let remote = target.starts_with("http://") || target.starts_with("https://");
    if remote && options.fakes.is_some() {
        return Err(WebFluentError::config(
            "--fakes builds the project with its fakes: give a route of it (`/users`), not a URL",
        ));
    }
    let mut faked: Option<PathBuf> = None;
    let (url, served) = if remote {
        (target.to_string(), None)
    } else {
        let route = if target.starts_with('/') {
            target.to_string()
        } else {
            format!("/{target}")
        };
        let (dir, config) = match options.fakes {
            // Built aside with the fakes, and thrown away after.
            Some(file) => {
                let config = crate::config::ProjectConfig::load(project_dir)?;
                let dir = super::verify::build_with_fakes(project_dir, file, json)?;
                faked = Some(dir.clone());
                (dir, config)
            }
            None => super::start::built_output(project_dir, allow_output_outside)?,
        };
        let base = config.build.base_path.trim_end_matches('/').to_string();
        let served = super::start::serve_built(dir, &base, "127.0.0.1:0", proxies);
        let served = match served {
            Ok(served) => served,
            Err(e) => {
                if let Some(dir) = &faked {
                    let _ = std::fs::remove_dir_all(dir);
                }
                return Err(e);
            }
        };
        (format!("{}{base}{route}", served.origin), Some(served))
    };
    let outcome = look(&url, options);
    if let Some(served) = served {
        served.close();
    }
    if let Some(dir) = faked {
        let _ = std::fs::remove_dir_all(dir);
    }
    let (report, visit) = outcome?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "url": target,
                "title": report["title"],
                "lang": report["lang"],
                "dir": report["dir"],
                "viewport": { "width": width, "height": height, "mobile": mobile },
                "outline": report["lines"],
                "problems": visit.errors,
                "screenshot": screenshot.map(|p| p.display().to_string()),
            }))
            .unwrap_or_default()
        );
        return Ok(());
    }
    let title = report["title"].as_str().unwrap_or_default();
    let lang = report["lang"].as_str().unwrap_or_default();
    println!(
        "{target} — \"{title}\"{} at {width}×{height}{}",
        if lang.is_empty() {
            String::new()
        } else {
            format!(" ({lang})")
        },
        if mobile { ", mobile" } else { "" }
    );
    for line in report["lines"].as_array().into_iter().flatten() {
        println!("  {}", line.as_str().unwrap_or_default());
    }
    if visit.errors.is_empty() {
        println!("no problems: nothing threw, logged an error or failed to arrive");
    } else {
        println!("{} problem(s):", visit.errors.len());
        for e in &visit.errors {
            println!("  {e}");
        }
    }
    if let Some(path) = screenshot {
        println!("screenshot: {}", path.display());
    }
    Ok(())
}

fn look(
    url: &str,
    options: &Look<'_>,
) -> Result<(serde_json::Value, crate::tools::browser::chrome::Visit)> {
    let mut browser = Browser::start()?;
    browser.set_viewport(options.width, options.height, options.mobile);
    // Done once it has loaded, fetched what it asked for and said nothing
    // for a quarter of a second; `--wait` is the most it is given.
    browser.settle_when_quiet(Some(250));
    let mut page = browser.open(url, options.wait_ms)?;
    let mut failed = None;
    for source in options.steps {
        let steps = crate::testing::parse_steps(source)
            .map_err(|e| WebFluentError::config(format!("--do {source}: {e}")))?;
        for step in &steps {
            let mut acting = Acting {
                browser: &mut browser,
                page: &mut page,
            };
            if let Err(why) = crate::testing::play_step(&mut acting, step)? {
                failed = Some(format!("--do {source}: {why}"));
                break;
            }
        }
        if failed.is_some() {
            break;
        }
    }
    let outline = browser.eval(
        &page,
        &OUTLINE.replace("__MAX__", &options.lines.to_string()),
    )?;
    let report: serde_json::Value =
        serde_json::from_str(outline.as_str().unwrap_or("{}")).unwrap_or_default();
    if let Some(path) = options.screenshot {
        let data = browser.screenshot(&page, options.full_page)?;
        let bytes = decode_base64(&data)
            .ok_or_else(|| WebFluentError::io("the browser's screenshot was not base64"))?;
        write_file(path, &bytes)?;
    }
    let mut visit = browser.close(page)?;
    if let Some(failed) = failed {
        visit.errors.insert(0, failed);
    }
    Ok((report, visit))
}

/// The open page, as a test's steps drive one.
struct Acting<'a> {
    browser: &'a mut Browser,
    page: &'a mut crate::tools::browser::chrome::Page,
}

impl crate::testing::Page for Acting<'_> {
    fn eval(&mut self, script: &str) -> Result<serde_json::Value> {
        self.browser.eval(self.page, script)
    }

    fn settle(&mut self, ms: u64) -> Result<()> {
        self.browser.settle(self.page, ms)
    }

    fn errors(&self) -> Vec<String> {
        self.page.errors().to_vec()
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| WebFluentError::io(format!("{}: {e}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|e| WebFluentError::io(format!("{}: {e}", path.display())))
}

/// Standard base64, as the browser sends a screenshot.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| -> Option<u32> {
        Some(u32::from(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        }))
    };
    let bytes: Vec<u8> = text
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut n = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            n |= value(c)? << (18 - 6 * i);
        }
        out.push((n >> 16) as u8);
        if chunk.len() > 2 {
            out.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            out.push(n as u8);
        }
    }
    Some(out)
}

/// Where `--screenshot` writes when it is given without a file: the
/// project's `.wf-cache/inspect/`, named by the route.
pub(crate) fn default_screenshot(project_dir: &Path, route: &str) -> PathBuf {
    let name: String = route
        .trim_matches('/')
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let name = if name.is_empty() {
        "index".to_string()
    } else {
        name
    };
    project_dir
        .join(".wf-cache/inspect")
        .join(format!("{name}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decodes_as_the_browser_encodes() {
        assert_eq!(decode_base64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(decode_base64("aGk=").unwrap(), b"hi");
        assert_eq!(decode_base64("YWJj").unwrap(), b"abc");
        assert!(decode_base64("a*b=").is_none());
    }

    #[test]
    fn a_screenshot_is_named_by_its_route() {
        let here = Path::new(".");
        assert!(
            default_screenshot(here, "/pokemon/pikachu")
                .ends_with(".wf-cache/inspect/pokemon-pikachu.png")
        );
        assert!(default_screenshot(here, "/").ends_with(".wf-cache/inspect/index.png"));
    }
}
