# Schema Diagnostic Codes

This reference lists the stable diagnostic codes returned during validation and documentation checking.

## DOC Diagnostics (Documentation Integrity)

| Code | Severity | Description |
|---|---|---|
| `DOC-001` | Fatal | Config domain lacks generated documentation facts. |
| `DOC-005` | Fatal | Missing domain description in configuration metadata. |
| `DOC-006` | Warning | Documented default values do not match schema. |
| `DOC-008` | Fatal | Receipt policy undocumented. |
| `DOC-010` | Fatal | Generated markdown projections are out-of-date. |
| `DOC-020` | Fatal | Digest mismatch inside the receipt binding wrapper. |

## SCH-JSON Diagnostics (JSON Schema Validation)

| Code | Severity | Description |
|---|---|---|
| `SCH-JSON-001` | Fatal | JSON Schema parsing feature disabled. |
| `SCH-JSON-002` | Fatal | JSON Schema parsing failed (invalid json format). |
| `SCH-JSON-008` | Fatal | Configuration validated against unadmitted schema. |
| `SCH-JSON-010` | Fatal | Schema cryptographic digest mismatch. |
| `SCH-JSON-015` | Error | Authority boundary is missing from JSON Schema. |
| `SCH-JSON-016` | Fatal | Unsupported JSON Schema dialect draft. |
