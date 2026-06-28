# Agent 01 Audit Report: Release Packaging, Tag, and Crates.io Readiness (v26.6.28)

**Audit Date:** June 28, 2026  
**Auditor:** Agent 01  
**Target Workspace:** [/Users/sac/star-toml](file:///Users/sac/star-toml)  
**Assigned Output Path:** [docs/audit/v26.6.28-adversarial/agent-01-release-packaging.md](file:///Users/sac/star-toml/docs/audit/v26.6.28-adversarial/agent-01-release-packaging.md)

---

## 1. Executive Summary

An audit of the repository tags, package versions, feature flags, test baseline, and packaging configuration of `star-toml` v26.6.28 was conducted. The audit reveals critical readiness failures that block a clean release to crates.io:

1. **Missing Gitignore and Packaging Pollution:** The repository lacks a `.gitignore` file. Consequently, `target/` directories containing compiler caches and build files are treated as untracked source files. Running `cargo package` fails because the workspace is dirty. Forcing packaging with `--allow-dirty` packages over 8,000 files, including compiled compiler artifacts under `target/`.
2. **Feature Compilation Defect:** Building/testing with `--features wasm4pm-compat` fails due to a mismatched type compilation error (`E0308`) in `src/bin/verifier_report.rs:368`.
3. **Invalid Command Syntax:** The requested command `cargo package --dry-run` is rejected by Cargo (`unexpected argument '--dry-run' found`) because `--dry-run` is a feature of `cargo publish`, not `cargo package`.

The Git tag `v26.6.28` successfully matches the `HEAD` commit, and the package versions declared in `Cargo.toml` and `star-toml-derive/Cargo.toml` correctly align with the tag.

---

## 2. Command Executions and Evidence

Below are the logs and evaluations for the required command executions in the workspace.

### 1. `git status --short`
```
?? target/debug/examples/validate-3211ae815cc1e8fd.9qxw5w19rbazbwr6y3a431v32.0ye607r.rcgu.o
?? target/debug/examples/validate-3211ae815cc1e8fd.azukm9ywi3grdtuj8edkdjyvv.01f367r.rcgu.o
... (hundreds of compile cache files under target/)
```
*Audit Observation:* Tracked source code is clean, but a massive number of untracked files are reported under `target/` due to the lack of a `.gitignore` file.

### 2. `git log --oneline -10`
```
3d4e994 fix: close 4 PARTIAL_ALIVE findings from independent audit
eea7e04 docs: add v26.6.28 admission receipt
ba0e8a2 chore: remove stale "deferred" doc comments now that AdmittedConfig/ConfigWitness exist
aab0113 feat: v26.6.28 — ST-102/106/108/109/111 admission substrate complete
800cb39 feat: WP-1/WP-2/WP-3 — typestate pipeline, provenance reports, traced merge
09714e9 chore: exclude agent scratch files and docs from crates.io package
1438c3f fix: add required metadata to star-toml-derive for crates.io publishing
e39c2ca feat: complete v26.6.27 implementation with typestate lifecycle, derive macro, and hardening
ad81ec7 feat: initial commit of standalone star-toml repository under praxis house style
```
*Audit Observation:* Total repository history consists of 9 commits. Commit `3d4e994` is the latest.

### 3. `git rev-parse HEAD`
```
3d4e9946e3b31827b14bff5d29f42288e8ffd7fd
```

### 4. `git rev-parse v26.6.28`
```
fe6e332738c870913cfd67f62981d967f3e9d554
```
*Audit Observation:* Resolving the annotated tag `v26.6.28` via `git rev-parse v26.6.28^{commit}` gives commit `3d4e9946e3b31827b14bff5d29f42288e8ffd7fd`, proving that the tag aligns perfectly with `HEAD`.

### 5. `rg -n '^version =|^name =|wasm4pm|wasm4pm-compat|blake3' Cargo.toml star-toml-derive/Cargo.toml Cargo.lock`
```
Cargo.toml:2:name = "star-toml"
Cargo.toml:3:version = "26.6.28"
Cargo.toml:21:blake3 = "1"
Cargo.toml:22:wasm4pm-compat = { version = "26.6", optional = true }
Cargo.toml:44:name = "verifier_report"
Cargo.toml:49:wasm4pm-compat = ["dep:wasm4pm-compat"]
star-toml-derive/Cargo.toml:2:name = "star-toml-derive"
star-toml-derive/Cargo.toml:3:version = "26.6.28"
Cargo.lock:3:version = 3
Cargo.lock:45:name = "blake3"
Cargo.lock:46:version = "1.8.5"
Cargo.lock:414:name = "star-toml"
Cargo.lock:415:version = "26.6.28"
Cargo.lock:417: "blake3",
Cargo.lock:423: "wasm4pm-compat",
Cargo.lock:427:name = "star-toml-derive"
Cargo.lock:428:version = "26.6.28"
Cargo.lock:581:name = "wasm4pm-compat"
Cargo.lock:582:version = "26.6.28"
Cargo.lock:586: "blake3",
```

### 6. `cargo test`
```
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.39s
```
*Audit Observation:* A total of 159 tests passed successfully under standard settings (including unit, adversarial, BRCE, validation macros, verifier checks, and doctests).

### 7. `cargo test --features wasm4pm-compat`
```
   Compiling star-toml v26.6.28 (/Users/sac/star-toml)
error[E0308]: mismatched types
   --> src/bin/verifier_report.rs:368:25
    |
368 |         let _unit: () = star_toml::ocel::export_events_to_ocel(&events);
    |                    --   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `()`, found `OcelLog`
```
*Audit Observation:* Building the `verifier_report` binary under the `wasm4pm-compat` feature fails compilation due to mismatched types.

### 8. `cargo package --dry-run`
```
error: unexpected argument '--dry-run' found
```
*Audit Observation:* This command syntax is invalid because `cargo package` does not support `--dry-run`. Running `cargo package` fails because of untracked changes, and using `cargo package --allow-dirty` attempts to package target directories.

---

## 3. Bound Index
- **B surface inspected:** Crate configuration parameters, manifest structure (`Cargo.toml` and `star-toml-derive/Cargo.toml`), and workspace declarations.
- **O surface inspected:** Repository files, git configuration, and target package structures.
- **μ surface inspected:** The packaging process of `cargo package` and feature gating on compilation.
- **C detectors inspected:** Git tags consistency, version alignment, dependency bounds (`wasm4pm-compat`, `blake3`), and test compliance.
- **W witnesses inspected:** Release manifests and Git tag object targets.
- **q evidence found:** Verification of commit alignment between tag `v26.6.28` and HEAD.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | Version bounds and library name definitions. | A root-level `.gitignore` file to establish repository boundaries. |
| **O** | Raw git history and manifest configurations. | None. |
| **O\*** | Tag commit pointer `3d4e994` and version definitions `26.6.28`. | None. |
| **μ** | Package construction step (`cargo package`) and feature compilation. | A correct return-type match in `export_events_to_ocel` when feature-enabled. |
| **A** | Source packages listed for distribution. | None. |
| **C** | Tag matching checks and dry run packages. | None. |
| **W** | Target git tag annotation matches HEAD. | None. |
| **q** | Standing score is failed because target files compile-fail on optional feature flags. | Complete build pass across all configured feature gating pathways. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | `cargo test` executes and passes all standard test cases. | None. | Core library baseline passes. |
| **Falsification** | `cargo test --features wasm4pm-compat` fails to compile, falsifying release packaging safety. | None. | Blocks build stability under optional features. |
| **Invariant** | Crate metadata has author, license, and keywords fields. | None. | Crate details are present. |
| **Determinism** | Version definitions are pinned and deterministic. | None. | Safe tag pointers. |
| **Provenance** | Crate tags and package definitions align to the same version number (`26.6.28`). | None. | Provenance matches. |
| **Witness** | Commit checksum matches the target release tag. | None. | Release tag authenticity is verified. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `tag_matches_head` | Yes | Yes (git checkout) | Yes | **PASS** |
| `version_matches_tag` | Yes | Yes (Cargo.toml check) | Yes | **PASS** |
| `dry_run_passes` | Yes | Yes (`cargo package`) | **No** (dirty check failure / invalid args) | **ACTIVE FAILURE** |
| `feature_compilation_passes` | Yes | Yes (features test) | **No** (verifier compilation E0308 error) | **ACTIVE FAILURE** |
| `only_intended_files_in_package` | Yes | Yes (list package) | **No** (packages untracked target/ files) | **ACTIVE FAILURE** |

---

## 5. Standing Decision

**Verdict:** **PARTIAL_ALIVE**  
*Rationale:* Although the repository tag correctly aligns with HEAD, and the package version mappings are identical, we cannot release `v26.6.28` due to compilation errors when enabling the `wasm4pm-compat` feature and massive package size pollution caused by the lack of a `.gitignore` file.

---

## 6. Detailed Audit Findings (Falsifier Evaluations)

### Falsifier 1: tag v26.6.28 does not match HEAD
* **Verdict:** **False**  
* **Analysis:** The annotated Git tag `v26.6.28` points to the tag object `fe6e332738c870913cfd67f62981d967f3e9d554`. Checking its underlying commit target via `git rev-parse v26.6.28^{commit}` returns commit `3d4e9946e3b31827b14bff5d29f42288e8ffd7fd`. This is the exact same commit hash returned by `git rev-parse HEAD`, confirming perfect tag alignment with `HEAD`.

### Falsifier 2: package version mismatch
* **Verdict:** **False**  
* **Analysis:** Ripgrep search reveals `Cargo.toml`, `star-toml-derive/Cargo.toml`, and the dependency specifications in `Cargo.lock` all state `version = "26.6.28"`. The workspace members are correctly locked to the same version, matching the release tag.

### Falsifier 3: build failure under wasm4pm-compat feature
* **Verdict:** **True**  
* **Analysis:** Activating the optional `wasm4pm-compat` dependency breaks the build. The method `export_events_to_ocel` in `src/ocel.rs` is defined to return `OcelLog` when the `wasm4pm-compat` feature is enabled. However, in `src/bin/verifier_report.rs` at line 368, the statement:
  ```rust
  let _unit: () = star_toml::ocel::export_events_to_ocel(&events);
  ```
  expects a unit type `()`, producing a `mismatched types` error (`E0308`) which terminates build compilation.

### Falsifier 4: cargo package includes build target artifacts
* **Verdict:** **True**  
* **Analysis:** Due to the absence of a `.gitignore` file in the project workspace, Cargo does not ignore files inside the `target/` directory. Running `cargo package --list --allow-dirty` lists compiled artifact objects (such as `.o`, `.rlib`, and `.d` files) under `target/` as files to include in the packaged tarball. This leaks thousands of unwanted build assets into the distribution package.
