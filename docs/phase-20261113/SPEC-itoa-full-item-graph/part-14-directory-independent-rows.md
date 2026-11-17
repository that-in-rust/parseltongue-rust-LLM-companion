# Part 14 — stored rows do not depend on the source directory

Back to the [index](README.md).

Status on 2026-11-18: **specified, not built.** Picked by the user on 2026-11-17 ("two
machines cannot compare or share a stored run").

**The defect, measured on 2026-11-18 at `40cf94ccf`.** The pinned source was copied to a
second directory and mapped from both. Of the exported rows, 0 of 172 node rows and 0 of
686 edge rows are equal, and the run ids differ. Replacing the directory text in the
export does not make them equal, because digests are computed over it.

```text
where the directory enters                          example
package id of the unit                              path+file:///home/u/a/itoa#1.0.18
path of a source file, in every identity            /home/u/a/itoa/src/lib.rs
digests computed over those two                     expansion provenance chain, the name
                                                    .parseltongue/macro-source/<digest>
run id                                              hashes the unit rows and file rows
```

**The fix.** The directory is replaced once, where it enters: the loader stores every
path under the workspace root with the fixed root `/parseltongue-workspace-root` in
place of the real one. Everything computed later is then equal by construction. The real
root is used only to open files.

**Not in this part.** The files of `core` are read from the toolchain directory, which
is under the home directory. No node or edge row holds such a path, but the run id
hashes their file rows, so the run id still differs between two home directories. On
one machine it is equal. Backlog.

## 1. Rust work mode

Spec Mode and Delivery Mode, with Reliability Mode for the path boundary. Component: the
loader (`parseltongue-crate-graph-loader`), one port of the syntax stage, the CLI, and a
pure rule in `parseltongue-core-row-types`.

Found with codebase-memory on 2026-11-18 (`trace_path`, inbound, depth 1) and read in
source:

- `load_workspace_crate_graph` runs `cargo metadata` and writes the unit rows (package
  id, source root, package root) and the row `workspace-config-state-records`, which
  holds the workspace root.
- `read_safe_source_bytes` opens a source file. Callers: `ingest_requested_source_file`
  and the sysroot loader. `ingest_requested_source_file` writes the row of
  `file-source-text-table`, keyed by the path. `ingest_workspace_source_files` lists
  directories.
- `SourcePathCanonicalPort::read_canonical_source_path` asks the disk whether a `mod`
  file exists. One caller, in the namespace input extraction.
- `compute_workspace_generation_digest` (the run id) hashes the payloads of
  `workspace-config-state-records`, `crate-dependency-graph-table` and
  `file-source-text-table`.

## 2. Executable requirements: REQ-RUST-072.0

```text
D1  WHEN the same source tree is mapped from two directories with the same executable and
    toolchain THEN the exported rows of item-effective-visibility-records and of
    edge-evidence-record-table SHALL be byte-equal, their header and run id included

D2  WHEN a run is published THEN no row of those two tables SHALL contain the text of the
    real workspace root

D3  WHEN a path is under the workspace root THEN its stored form SHALL be the fixed root
    /parseltongue-workspace-root followed by the path relative to the real root. A path
    that only shares a name prefix with the root (/a/itoa2 for the root /a/itoa) SHALL be
    left as it is

D4  WHEN a path is outside the workspace root THEN its stored form SHALL be the path
    itself

D5  WHEN a stored path is opened THEN the file SHALL be the one under the real workspace
    root; restoring the stored form of a path SHALL give the path back

D6  WHEN the workspace root is given through a symbolic link or with a final slash THEN
    the root that is replaced SHALL be its canonical form

D7  WHEN a path under the real workspace root starts with the fixed root itself THEN the
    run SHALL fail with a typed error and store nothing

A1  WHEN the mapper runs on pinned itoa THEN the stored graph SHALL equal both frozen
    expected graphs, with no change to an expected file
```

## 3. Design (L1 / L2 / L3)

- **L1, `parseltongue-core-row-types`.** `WorkspaceRootRelocationValue` holds the
  canonical real root. Two pure functions: one gives the stored form of a path or of a
  package id (D3, D4), one gives the disk path of a stored form (D5). Both work on path
  components, not on text prefixes. A typed error for D7.
- **L3, `parseltongue-crate-graph-loader`.** The output of `cargo metadata` is rewritten
  once, before any row is built from it: every string that is a path or a `path+file://`
  package id under the root gets the stored form. `read_safe_source_bytes` and the
  directory listing take the relocation value, restore the disk path to open the file,
  and return the stored form as the row key.
- **L2, the syntax stage.** `NativeCanonicalSourcePort` restores the disk path before it
  asks the disk, and returns the stored form.
- **CLI.** Canonicalises the root argument (D6), builds the value and passes it down.

No new table or stage. The expected graph names rows by label and by a path relative to
the root, so it does not change (A1).

## 4. Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| D3 D4 D5 D7 | `test_workspace_root_path_relocation` | unit, core | stored form and restore for a file, the root itself, a package id, a name-prefix sibling, an outside path, and the fixed root inside the tree | 0 wrong |
| D6 | `test_symlinked_root_rows_equal` | real source | a run through a symbolic link stores the rows of a run on the directory | 0 differing rows |
| D1 D2 | `test_itoa_rows_directory_independent` | real source | two copies of pinned itoa in two directories export equal node and edge rows; neither directory's text is in them | 0 differing rows |
| A1 | `test_itoa_declarations_graph_equality`, `test_itoa_whole_graph_equality` | real source | stored graph equals the frozen expected graph | 0 differing rows |

## 5. Implementation plan

1. `test_itoa_rows_directory_independent`, seen failing on the measured difference.
2. Core: the value, the two functions and the error; unit test first.
3. Loader, syntax port and CLI; the real-source test passes.
4. Real-source tests that build an identity from the temporary directory use the fixed
   root.
5. Gates, a timed run, the documents and the log.

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

- **The toolchain directory.** A second fixed root for the sysroot would make the run id
  equal across home directories. Backlog.
- **A workspace with a member outside the root.** Its paths stay absolute (D4), so its
  rows still depend on the directory. itoa has none.
- **Stored runs made before this part** hold real paths and get a different run id. No
  reader compares across that boundary.
