//! Warm renders through WebFluent's Rust API: the template is parsed and
//! checked once, then `render_pdf` runs over the same data again and again,
//! as a server rendering invoices on request would.
//!
//! ```text
//! wf-pdf-bench <template.wf> <data.json> <out.pdf> [renders] [warmup]
//! ```
//!
//! Prints one JSON object: the time to build the template, every measured
//! render's time in milliseconds, and the size of the PDF.

use std::time::Instant;

use webfluent::Template;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: wf-pdf-bench <template.wf> <data.json> <out.pdf> [renders] [warmup]");
        std::process::exit(2);
    }
    let renders: usize = args.get(4).map_or(100, |n| n.parse().expect("renders"));
    let warmup: usize = args.get(5).map_or(5, |n| n.parse().expect("warmup"));

    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).expect("read the data"))
            .expect("parse the data");

    let start = Instant::now();
    let template = Template::from_file(&args[1]).expect("build the template");
    let setup_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut pdf = Vec::new();
    for _ in 0..warmup {
        pdf = template.render_pdf(&data).expect("render");
    }
    let mut times = Vec::with_capacity(renders);
    let all = Instant::now();
    for _ in 0..renders {
        let t = Instant::now();
        pdf = template.render_pdf(&data).expect("render");
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let total_ms = all.elapsed().as_secs_f64() * 1000.0;
    std::fs::write(&args[3], &pdf).expect("write the PDF");

    let times: Vec<String> = times.iter().map(|t| format!("{t:.3}")).collect();
    println!(
        "{{\"setup_ms\":{setup_ms:.3},\"total_ms\":{total_ms:.3},\"warmup\":{warmup},\"bytes\":{},\"times_ms\":[{}]}}",
        pdf.len(),
        times.join(",")
    );
}
