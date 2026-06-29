# WASM ABI Interface Reference

This reference documents the WASM-exported functions compiled from the `star-toml-wasm` crate.

## Native to WASM ABI Map

| Function | Execution on Wasm32 Target | Result Shape |
|---|---|---|
| `validate_schema` | Returns `WASM-BRIDGE-UNAVAILABLE` stub | JSON string |
| `validate_config` | Performs TOML parse validation (no bridge checks) | JSON string |
| `validate_json_schema_bridge` | Returns stub value | JSON string |
| `generate_docs_metadata` | Stub placeholder | JSON string |
| `verify_receipt` | Complete cryptographical validation | JSON string |
| `explain_diagnostic` | Returns human-readable diagnostic messages | String |

## JSON Result Structure

Successful validation results shape:
```json
{
  "status": "ADMITTED",
  "diagnostics": [],
  "digests": {
    "schema": "ad60c...",
    "config": "3491f..."
  },
  "receipts": []
}
```
