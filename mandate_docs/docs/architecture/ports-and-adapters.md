# Ports and adapters

This is the architecture overview of the mandate parser crate's internals,
for a reader who wants the file-by-file shape and the driven ports'
contracts. See [`mandate-parser-overview.md`](mandate-parser-overview.md)
for the crate's overall shape.

## The pieces

| File | Layer | Role |
|---|---|---|
| `src/domain/model/mandate.rs` | domain | Plain data types for a mandate: `Mandate`, `Rule`, `RuleKind`, `GovernedDoc`, `CodeLink`. No serde, no I/O. |
| `src/domain/model/file_tree.rs` | domain | `FileTreeSnapshot`, a value recording every file and directory under a root at one point in time. `EntryKind` distinguishes a file from a directory. |
| `src/domain/model/validation.rs` | domain | `validate`, `ValidationReport`, `ValidationError`, `ValidationWarning`. Checks a `Mandate` against a `&FileTreeSnapshot`. |
| `src/domain/model/run_report.rs` | domain | `RunReportMandatesValidation`, the report for one run: root, snapshot entry count, warnings, and one outcome per mandate. `is_valid()`. |
| `src/domain/ports/driven/file_tree_source.rs` | port | `FileTreeSource`, snapshots a root and cheaply checks whether one directory holds another. |
| `src/domain/ports/driven/mandate_store.rs` | port | `MandateStore`, lists and reads mandate files under a root. |
| `src/domain/ports/driven/mandate_parser.rs` | port | `MandateParser`, turns mandate text into a `Mandate`. |
| `src/domain/usecases/run_mandates.rs` | domain | `RunMandates`, the use case: discovers the project root, takes the snapshot, selects mandates, parses and validates each, and fills the report. `RunError` for its failure modes. |
| `src/adapters/driven/file_system/mod.rs` | adapter-internal seam | `FileSystem` trait with `list_dir`, `read_to_string`, `is_dir`, `exists`; `DirEntry`; `FileSystemError`. |
| `src/adapters/driven/file_system/os_file_system.rs` | adapter-internal seam | `OsFileSystem`, the real disk implementation. |
| `src/adapters/driven/file_tree/fs_file_tree_source.rs` | driven adapter | `FsFileTreeSource<F>`, generic over `FileSystem`, a `FileTreeSource` that walks the filesystem. |
| `src/adapters/driven/mandate_store/fs_mandate_store.rs` | driven adapter | `FsMandateStore<F>`, generic over `FileSystem`, a `MandateStore` that lists and reads `.yaml` files under `<root>/.mandate/mandates/`. |
| `src/adapters/driven/mandate_parser/yaml_mandate_parser.rs` | driven adapter | `YamlMandateParser`, a `MandateParser`. Turns mandate YAML text into a `Mandate`. |
| `src/adapters/driving/cli/mod.rs` | driving adapter | `parse_args` turns the process argument list into an `Invocation`. `validation_lines`, `report_lines` and `render` produce and write the report text. `run` executes the use case and writes the report or error. Neither the adapter nor its tests do any I/O other than to the provided writers; `main.rs` wires the adapters and maps `report.is_valid()` to the exit code. |
| `src/main.rs` | composition root | Collects the process arguments, builds the real adapters, runs `RunMandates`, and prints and exits according to the report. |
| `src/lib.rs` | composition root | Declares the `adapters` and `domain` modules. |

Dependencies point one way: the adapters, the domain/usecases and the
composition root import the rest of the domain; the domain imports nothing outside
`std`. `FileTreeSource`, `MandateStore` and `MandateParser` are driven ports
because `RunMandates` calls them to reach the filesystem and the YAML
parser; `src/adapters/driving/cli/mod.rs` is the sole driving adapter, since nothing
else calls into the crate yet.

The `serde`-derived structs (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`,
`CodeLinkDoc`) live only in `src/adapters/driven/mandate_parser/yaml_mandate_parser.rs` and never leave that
module. `parse_mandate` maps each one into the plain domain types in
`src/domain/model/mandate.rs` before returning. This follows the ports and
adapters rule that no library type may enter the domain: the domain types
carry no `serde` attributes and would compile unchanged if the YAML
library were replaced.

## The ports

`FileTreeSource` has two methods:

```rust
fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError>;
fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError>;
```

`snapshot` walks the whole tree under `root`. `has_directory` answers
whether `dir` contains a directory entry named `name`, without building a
snapshot, and answers `false` rather than erroring when `dir` itself
cannot be read, so an unreadable ancestor is a reason it cannot be the
root and never a reason to stop looking further up. `FsFileTreeSource<F>`
is generic over `FileSystem` and is the sole adapter implementation.

`MandateStore` has two methods:

```rust
fn list(&self, root: &Path) -> Result<Vec<String>, StoreError>;
fn read(&self, root: &Path, file_name: &str) -> Result<MandateFile, StoreError>;
```

The port also defines `MANDATE_DIR` (".mandate") and `MANDATES_DIR`
(".mandate/mandates"), used by the adapter and the use case. `list` returns the
file names of the `.yaml` files directly in `<root>/.mandate/mandates/`,
sorted; `read` returns one file's text. `FsMandateStore<F>` is generic over
`FileSystem` and is the sole adapter implementation.

`MandateParser` has one method, turning mandate text into a `Mandate` or a
`MandateParseError`; `YamlMandateParser` in
`src/adapters/driven/mandate_parser/yaml_mandate_parser.rs` is the sole adapter.
`MandateParseError` is a domain type declared on the port, so the YAML crate's
internal error never leaves the adapter. It is described in full in
[`../reference/mandate-format-checks.md`](../reference/mandate-format-checks.md).

## The file system seam

`FileSystem` is not a port the domain calls; it is an adapter-internal seam
that lets the driven adapters abstract away filesystem calls. This lets
fixture tests run the real `FsFileTreeSource` and `FsMandateStore` adapters
over a fake in-memory implementation (`FakeVirtualMachine`) without touching
disk, proving the adapters' real logic against fixture data. See
[`testing.md`](testing.md) for how the fixtures use it.

`FakeVirtualMachine` in `tests/fixtures/support.rs` implements `FileSystem`,
holding paths and mandate texts. A fixture builds the real adapters over the
fake: `FsFileTreeSource::new(fake_fs)` and `FsMandateStore::new(fake_fs)`.
Because the adapters see the same `FileSystem` interface in tests and in
production, the real adapters are exercised on fake data, and their behaviour
is covered without test doubles.

Each port's doc comment states its contract: `snapshot` path format and
symlink policy, `has_directory` cheapness, `read` bare-file-name rule, and
the error types on the parser and store ports.
