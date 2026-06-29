# star-toml Configuration Reference

* **Model Version**: `26.6.29`
* **Generated At**: `2026-06-28T15:41:36-07:00`
* **Input Facts Digest**: `44d4f616d23f897f7718aea799cafe0d5190253db28dfd0d816eb18128c33815`
* **Model Digest**: `4c9e73f1727cc0abae02b1a3fa030c9fa89237150bc25ab848b4e718a84fc10e`

## 🏛️ Receipts & Standing

| Admitted Config Digest | Model Digest | Projection Digest | Generator Version | Signer/Operator |
|---|---|---|---|---|
| `44d4f616d23f897f7718aea799cafe0d5190253db28dfd0d816eb18128c33815` | `4c9e73f1727cc0abae02b1a3fa030c9fa89237150bc25ab848b4e718a84fc10e` | `35c714fa7a876dc8e7bb186e1170ce2017cbd5c344767a11b59aa50840473709` | `26.6.29` | `release-verifier-signer` |

## Domain: Star TOML System Config (system)

* **Stability**: `stable`
* **Description**: Configuration schema governing star-toml sandbox policies, log paths, and cache sizes.

### 🛡️ Authority Boundary

* **Allowed Control**: `["cache_size", "log_path"]`
* **Forbidden Claims**: `["root_authority"]`
* **Delegates To**: `platform-ops`

### ⚙️ Fields Reference

| Field | Type | Required | Default | Allowed Values | Forbidden Values | Description | Stability |
|---|---|---|---|---|---|---|---|
| `cache_size` | `integer` | `false` | `10485760` | `-` | `-` | Maximum memory cached configuration instances in bytes (default: 10485760). | `stable` |
| `log_path` | `String` | `true` | `-` | `-` | `-` | Absolute or relative path to write lifecycle events (required). | `stable` |

### 🔍 Validators

* **`range_i64`**: must be in range 1024..=1073741824 (Applies to: `["cache_size"]`, Severity: `Error`)
* **`non_empty`**: must not be empty (Applies to: `["log_path"]`, Severity: `Error`)
* **`required`**: field is required (Applies to: `["log_path"]`, Severity: `Error`)

### ❌ Counterexamples

* **`empty`** - *log_path* (Detection: `validation_fails`, Severity: `Error`)
  * **Repair**: provide a non-empty string
* **`missing`** - *log_path* (Detection: `validation_fails`, Severity: `Error`)
  * **Repair**: provide the required field
* **`out_of_range`** - *cache_size* (Detection: `validation_fails`, Severity: `Error`)
  * **Repair**: provide a value between 1024 and 1073741824

