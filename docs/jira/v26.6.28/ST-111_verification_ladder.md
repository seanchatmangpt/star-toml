# ST-111: BRCE Verification Ladder & Verifier Report

## Description
Implements the comprehensive Bounded Receipted Chatman Equation (BRCE) Verification Ladder and the Verifier Report generator. Bounded standing requires that observations, invariants, counterexamples, witnesses, replay capabilities, and deterministic transformations align. This ticket structures the 26 specific testing requirements to ensure the admissibility of config as project law.

## Admissibility Definition
$$O^*_{config} = 1 \iff Parse = 1 \wedge LayerMerge = 1 \wedge Deserialize = 1 \wedge Validate = 1 \wedge PathSafe = 1 \wedge DeterministicRewrite = 1 \wedge Witness = 1 \wedge CounterexampleSet = \emptyset$$
$$q_{config} = 1 \iff C_{config}(x_t) = \emptyset \wedge W_{config} \neq \emptyset$$
$$V_{star-toml,26.6.28} = 1 \iff \text{failset\_cardinality} = 0$$

## Key Requirements (The 26 BRCE Test Specifications)

1. **Truth Test**: Verify that a known valid, fully populated TOML string loads successfully and generates `q_config = 1`.
2. **Falsification Test**: Verify that a syntactically invalid TOML file fails immediately with `q_config = 0` and does not progress to type validation.
3. **Counterfactual Test**: Demonstrate causal sensitivity by modifying a single environment variable and verifying that the value source and final `ConfigWitness` change, while admissibility `q_config = 1` remains constant.
4. **Invariant Test**: Verify that every validation failure always produces a structure containing `loc`, `code`, `severity`, `input`, `message`, and `repair_hint`.
5. **Metamorphic Test**: Assert that reordering independent tables or inserting non-semantic whitespace in a raw TOML string does not change the generated canonical digest.
6. **Boundary Test**: Verify parameter limit bounds (e.g., maximum allowed port number `65535` passes, but `65536` triggers a validation error).
7. **Conservation Test**: Assert that every field in the final `AdmittedConfig<T>` has a winning source layer marked in the metadata.
8. **Determinism Test**: Verify that loading from identical inputs (files, environment variables, defaults) multiple times generates the exact same `ConfigWitness`.
9. **Idempotence Test**: Verify that running the serialization chain twice yields the same output: `save_canonical(load_admitted(x)) == save_canonical(load_admitted(save_canonical(load_admitted(x))))`.
10. **Replay Test**: Verify that a `ConfigWitness` can be deterministically recomputed from the aggregated sources, layers, environment overrides, validation results, and canonical outputs.
11. **Provenance Test**: Verify that every active environment override reports its `raw_env_key`, `mapped_path`, `coerced_type`, and `value_digest`.
12. **Path Safety Test**: Assert that path validation fails (`q_config = 0`) when traversals (`..`), null bytes (`\0`), forbidden absolute paths, or source escapes are present.
13. **Type-state Test**: Assert at compile-time/runtime that a configuration cannot trigger a write operation (`save_canonical`) before completing semantic validation.
14. **Unknown-field Test**: Assert that unrecognized policy-like keys cause load failures in trusted mode.
15. **Layer-order Test**: Assert that the precedence `Defaults < Files < Env` is preserved, reproducible, and reported.
16. **Array Replacement Test**: Verify that higher-priority layers replace arrays entirely rather than merging or appending elements.
17. **Table Merge Test**: Assert that tables in config files merge recursively key-by-key.
18. **Scalar Replacement Test**: Assert that higher-priority scalars override lower-priority scalars.
19. **Env Coercion Test**: Verify that string environment variables are deterministically coerced to Booleans, Integers, Floats, or Strings.
20. **Error Topology Test**: Assert that the `variant_id` computed from FNV-1a hashing of sorted `loc:code` is identical for different inputs that trigger the same failure pattern.
21. **Fitness Test**: Validate that `fitness = PassedChecks / TotalChecks` operates as a conformance score but does *not* grant standing if any fatal error is present.
22. **Repair Test**: Verify that `repair_hint` generates minimum edit suggestions but does *not* modify admissibility standing without re-validation.
23. **Macro Test**: Verify that `#[derive(Validate)]` generates valid Rust validation checks without generating standing evidence by itself.
24. **Schema Test**: Assert that the `schema!` macro validates raw TOML shape, but does not prove downstream safety by itself.
25. **Canonicalization Test**: Assert that canonical serialization produces key-sorted machine TOML and does not claim comment preservation unless proven.
26. **Comment-preservation Verification**: Set up a test proving that a comment-preservation edit does not lose structural keys. A claim of comment preservation without a validated test suite is treated as a active counterexample.

## Verifier Report Generator
- Implement a test command/binary `cargo run --bin verifier_report` that evaluates the 22 core counterexamples and outputs a `VERIFIER_REPORT.md` file.
- The generator must fail and output a non-zero exit code if any counterexample detector fails to trigger on adversarial inputs.

## Acceptance Criteria
- [ ] Implement all 26 test specifications under a `tests/brce/` directory.
- [ ] The `verifier_report` binary runs clean and produces the verification markdown report showing all 22 counterexamples are mitigated.
- [ ] No application or downstream policies are compiled in `star-toml`.

## Verification Method
- **Execution**: Run `cargo test --test brce_ladder` and `cargo run --bin verifier_report`.
- **Artifact Verification**: Confirm that `VERIFIER_REPORT.md` is generated with `failset_cardinality = 0`.
