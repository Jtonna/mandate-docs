//! Fixture-level tests for the CLI adapter's `run`: it executes the use
//! case, renders the report or the error to the given writers, and hands
//! the report back without ever deciding an exit code itself.

use std::path::Path;

use mandate::adapters::driven::file_tree::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::driven::mandate_parser::yaml_mandate_parser::YamlMandateParser;
use mandate::adapters::driven::mandate_store::fs_mandate_store::FsMandateStore;
use mandate::adapters::driving::cli::{self, Invocation};
use mandate::domain::usecases::run_mandates::RunMandates;

use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn run_writes_the_report_to_out_and_returns_it() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate).with_mandate("a.yaml", MANDATE);

    let source = FsFileTreeSource::new(&vm);
    let store = FsMandateStore::new(&vm);
    let parser = YamlMandateParser;
    let run_mandates = RunMandates::new(&source, &store, &parser);
    let invocation = Invocation {
        root: None,
        mandates: Vec::new(),
    };

    let mut out = Vec::new();
    let mut err = Vec::new();
    let report = cli::run(
        &invocation,
        Path::new("/repo"),
        &run_mandates,
        &mut out,
        &mut err,
    );

    let report = report.expect("expected a report");
    assert!(report.is_valid());
    assert!(err.is_empty(), "err: {:?}", String::from_utf8(err));

    assert_eq!(
        String::from_utf8(out).expect("utf8"),
        "repository: /repo\n\
         snapshot: 29 entries\n\
         \n\
         a.yaml\n\
         \x20 valid: 3 rules, 2 documents, 11 source files\n\
         \n\
         1 mandates checked, 0 invalid\n"
    );
}

#[test]
fn run_writes_an_unknown_mandate_error_to_err_and_returns_none() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate).with_mandate("a.yaml", MANDATE);

    let source = FsFileTreeSource::new(&vm);
    let store = FsMandateStore::new(&vm);
    let parser = YamlMandateParser;
    let run_mandates = RunMandates::new(&source, &store, &parser);
    let invocation = Invocation {
        root: None,
        mandates: vec!["missing.yaml".to_string()],
    };

    let mut out = Vec::new();
    let mut err = Vec::new();
    let report = cli::run(
        &invocation,
        Path::new("/repo"),
        &run_mandates,
        &mut out,
        &mut err,
    );

    assert!(report.is_none());
    assert!(out.is_empty(), "out: {:?}", String::from_utf8(out));
    assert_eq!(
        String::from_utf8(err).expect("utf8"),
        "unknown mandate 'missing.yaml'; available mandates are: a.yaml\n"
    );
}
