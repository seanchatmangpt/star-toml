# star-toml Verifier Report

**Total**: 22  **Passed**: 22  **Failed**: 0

| # | Counterexample | Status | failset_cardinality |
|---|----------------|--------|--------------------|
| 1 | parse_valid_treated_as_trusted | PASS | 0 |
| 2 | implicit_source_used | PASS | 0 |
| 3 | missing_required_file_not_error | PASS | 0 |
| 4 | ambiguous_layer_order | PASS | 0 |
| 5 | unreported_layer_override | PASS | 0 |
| 6 | env_override_without_prefix | PASS | 0 |
| 7 | env_override_not_reported | PASS | 0 |
| 8 | unknown_field_accepted_in_trusted_mode | PASS | 0 |
| 9 | validation_not_run | PASS | 0 |
| 10 | validation_error_without_path | PASS | 0 |
| 11 | fatal_error_downgraded | PASS | 0 |
| 12 | path_traversal_accepted | PASS | 0 |
| 13 | null_byte_path_accepted | PASS | 0 |
| 14 | source_relative_path_unresolved | PASS | 0 |
| 15 | nondeterministic_save | PASS | 0 |
| 16 | comment_preservation_claim_unproven | PASS | 0 |
| 17 | rewrite_without_validation | PASS | 0 |
| 18 | witness_missing_source_digest | PASS | 0 |
| 19 | witness_missing_env_report | PASS | 0 |
| 20 | witness_missing_validation_report | PASS | 0 |
| 21 | witness_nondeterministic | PASS | 0 |
| 22 | downstream_policy_inside_star_toml | PASS | 0 |
