# Fixture tests build real adapters over one fake, in one test target

Status: accepted

## Context

Testing the filesystem-facing adapters needs a way to run their logic
without touching disk. A fake that reimplements a driven port directly, as
opposed to faking the layer below it, can drift from the real adapter's
behaviour; this happened once in this crate. Validation behaviour also
needs many small, independently readable cases.

## Decision

The filesystem adapters (`FsFileTreeSource`, `FsMandateStore`) share an
adapter-internal `FileSystem` seam, so fixture tests build the real
adapters over one in-memory fake (`FakeVirtualMachine`) instead of faking
the ports themselves. Validation and run behaviour are proven by fixture
cases inside one test target, each a directory holding its own mandate and
its own tests, built on that one shared fake.

## Consequences

Fixture tests exercise the real, production adapter and use case logic
against fake data, so their behaviour is covered without any test double
standing in for adapter logic. A new case is cheap to add: a new directory
with a `mandate.yaml` and a `mod.rs`, registered with one `mod` line, and a
single case can prove several things at once.
