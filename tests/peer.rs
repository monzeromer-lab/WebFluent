//! `peer link = rtc(…)`: a WebRTC data channel, as the compiler reads and
//! emits it. What it does between two real pages — connecting, a message
//! each way, one side seeing the other leave — is `tests/browser/peer.mjs`.

mod common;

use common::spa_generated;
use webfluent::parse_source;

fn page(body: &str) -> String {
    format!("page P(path: \"/\", title: \"T\", description: \"D\") {{\n{body}\n}}\n")
}

const SIGNALLED: &str = r#"
    state got = ""
    channel room = broadcast("room") { on message(m) { link.signal(m) } }
    peer link = rtc(signal: m => room.post(m), initiator: true, ice: [{ urls: "stun:stun.example.com" }]) {
        on message(m) { got = m.text }
    }
    Heading("T").h1
    match link {
        connecting { Text("…") }
        open { Button("Send") { on click { link.send({ text: "hi" }) } } }
        closed(c) { Text("gone") }
        error(e) { Text(e.message) }
    }
"#;

#[test]
fn a_peer_is_opened_with_its_options_and_no_address() {
    let js = spa_generated(&page(SIGNALLED));
    let at = js.find("WF.rtc(").expect("the peer is opened");
    let call = &js[at..at + js[at..].find(";\n").unwrap_or(200)];
    assert!(
        call.starts_with("WF.rtc({"),
        "a peer's one argument is its options: {call}"
    );
    for want in ["signal:", "initiator:", "ice:", "onMessage:"] {
        assert!(call.contains(want), "{want} in {call}");
    }
}

#[test]
fn a_page_with_a_peer_carries_the_module_and_one_without_does_not() {
    let with = common::raw_output(common::Backend::Spa, &page(SIGNALLED));
    assert!(
        with.contains("function rtc("),
        "the peer module is built in"
    );
    let without = common::raw_output(common::Backend::Spa, &page("Heading(\"T\").h1"));
    assert!(
        !without.contains("function rtc("),
        "and left out where nothing opens one"
    );
}

#[test]
fn a_peer_without_signal_is_refused_with_how_to_write_it() {
    let err = parse_source(&page("peer link = rtc(initiator: true)"), "t.wf")
        .expect_err("refused")
        .to_string();
    assert!(err.contains("needs `signal:`"), "{err}");
}

#[test]
fn a_peer_takes_no_address() {
    assert!(
        parse_source(
            &page(r#"peer link = rtc("wss://x", signal: m => log(m))"#),
            "t.wf"
        )
        .is_err(),
        "an address is for a socket; a peer is reached through its signalling"
    );
}
