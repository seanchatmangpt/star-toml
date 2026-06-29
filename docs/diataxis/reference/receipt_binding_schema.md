# Receipt Binding Schema Reference

This reference details the cryptographical structure of admission receipts.

## Receipt Fields

| Field Name | Type | Description |
|---|---|---|
| `json_schema_digest` | String | BLAKE3 hash value of the JSON Schema. |
| `admitted_schema_digest` | String | Verification digest matching the admitted pipeline registry. |
| `admitted_config_digest` | String | Hash of canonical configuration TOML structure. |
| `documentation_model_digest` | String | BLAKE3 hash of compiled domain model metadata. |
| `projection_digest` | String | Hash of exported markdown file output. |
| `generator_version` | String | Version value matching the compiler tool (e.g. `26.6.29`). |
| `signer` | String | The operator key signing the admission status. |

## Bootstrapping Avoidance Policy

To prevent circular dependency issues during model generation:
- The hash calculation for `compute_model_digest()` excludes `receipts` and `domain.receipt_binding` fields from the cryptographic hash inputs.
