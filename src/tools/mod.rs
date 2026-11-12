//! The tools built on the compiler: migrate, test, verify, audit, studio,
//! agent, init, generate, docs and the browser they drive.

/// What an agent works through: a change set held apart from the working
/// tree, built before it is read, and the context it is given.
pub mod agent;
/// `wf audit`: what a project trusts — markup it did not write, other
/// origins, what it keeps on the reader's machine, its `env` names, its
/// policy and its dependencies.
pub mod audit;
pub mod browser;
pub mod commands;
/// `wf docs`: the component gallery, one self-contained HTML page.
pub mod gallery;
/// `wf generate` as a library call: the source of a new page, component or
/// store, and [`generate::create`] to write one in the project's layout.
pub mod generate;
/// `wf init`: a new project from a template.
pub mod init;
/// A project built from memory, with nothing read from or written to a disk:
/// what the browser's playground runs.
pub mod memory;
/// `wf migrate`: the original grammar rewritten as WebFluent 3.
pub mod migrate;
/// What a running page changes to follow an edit, with no reload.
pub mod patch;
pub mod registry_json;
pub mod registry_schema;
/// Studio integration facade — [`studio::compile_studio`] turns a program into a
/// [`studio::CompiledSite`] (SSG pages + CSS + JS + node map) for the preview.
pub mod studio;
/// `wf test`: a project's tests, rendered or played in a browser, and what
/// each came to.
pub mod testing;
pub mod theme_view;
/// `wf verify`: every page of a built project in a browser — what each
/// route did, and the built-ins no page drew.
pub mod verify;
