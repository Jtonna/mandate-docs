//! Validation fixtures. Each case is a directory beside this
//! file holding a `mod.rs` and its own `mandate.yaml`. Adding a case is
//! creating that directory and adding its `mod` line below; nothing else
//! needs to be touched.

mod support;

mod all_links_present;
mod case_mismatch;
mod code_missing;
mod doc_missing;
mod duplicate_rule;
mod no_rules;
mod run_all;
mod run_from_subdirectory;
mod run_named;
mod run_no_mandates;
mod run_unknown_name;
mod ungoverned_doc;
mod unknown_rule;
mod unreferenced_rule;
