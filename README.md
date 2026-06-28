# star-toml

A framework for loading, layering, validating, and **admitting** configuration files in Rust — the `*` in `*.toml`.

Most configuration frameworks parse a file into a struct and stop. `star-toml` treats configuration as **operational law** and provides the core substrate for **config admission**.

```text
Do not build a path.
Build the bounded space of paths,
the laws that admit them,
the generators that manufacture them,
the detectors that falsify them,
the witnesses that preserve them,
and the gate that decides standing.
```

---

## 🏛️ The Core Thesis: Configuration is Operational Law

Most software systems do not fail because they lack code. They fail because their operational law is implicit, underbounded, or non-witnessed.

`star-toml` replaces **narrative authority** ("trust me, it works") with **machine-visible standing**:
* **Raw Parse ≠ Trusted Config:** Syntactically valid TOML is not an admitted configuration.
* **OCEL = Lifecycle/Process History:** Process-mining logs (`wasm4pm-compat` OCEL export) record the process history but do not compute $q_{config}$ or grant standing.
* **q_config = Standing Decision:** The configuration standing bit ($q_{config} = 1$) is calculated from pipeline completeness, witness integrity, and an empty failset.
* **AdmittedConfig<T>:** The terminal, immutable, witness-backed envelope representing a configuration with standing.

---

## 🚀 The Config Admission Pipeline

`star-toml` enforces a strict, typestate-guaranteed admission pipeline:

```text
Raw (String/File)
 ↳ BoundedSources (Source list checked)
    ↳ Merged (Traced layer merge)
       ↳ EnvResolved (Prefixed environment variables)
          ↳ Deserialized<T> (Schema alignment)
             ↳ Validated<T> (Pydantic-grade custom rules)
                ↳ Frozen<T> (Immutable representation)
                   ↳ CanonicalSaved (Alphabetically sorted, deterministic serialization)
                      ↳ Witnessed (BLAKE3 hash binding all provenance inputs)
                         ↳ AdmittedConfig<T> (Terminal admitted config)
```

---

## 🛠️ Usage Example

```rust
use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AppConfig {
    name: String,
    port: u16,
}

impl Validate for AppConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
    }
}
impl ConfigLifecycle for AppConfig {}

fn main() {
    // 1. Establish the admission loader with layers and env policies
    let loader = TrustedLoader::new()
        .layer_file("config.toml")
        .env_prefix("APP_");

    // 2. Load the terminal admitted config wrapper
    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            println!("Admission Granted ($q_config = 1$)!");
            println!("Config: {:?}", admitted.value());
            println!("Witness: {}", admitted.witness().hash());
        }
        Err(e) => {
            println!("Admission Refused ($q_config = 0$): {:?}", e);
        }
    }
}
```

---

## 📐 DfCM (Design for Combinatorial Maximalism)

`star-toml` is designed under the **DfCM** discipline. MVP asks: *"What is the smallest happy path?"* DfCM asks: *"What is the complete bounded variant space this system must survive?"*

Every configuration loader evaluates:
1. **Sources:** Inline, required file, optional file, environment.
2. **Layers:** Defaults $\rightarrow$ base $\rightarrow$ profile $\rightarrow$ env.
3. **Paths:** Validated against Sandbox, RelativeOnly, or BlockForbidden policies.
4. **Validation:** Invariant rules with error or fatal severities.

---

## 🔍 Verification & Conformance

`star-toml` includes a built-in verifier that checks for active counterexamples (such as directory traversals, unknown fields accepted in trusted mode, and validation errors without paths) to ensure the release has absolute conformance.

Run the verifier report:
```bash
cargo run --bin verifier_report
```

Run DfCM examples:
```bash
cargo run --example basic_admitted_config
cargo run --example layered_profiles
cargo run --example env_overrides
cargo run --example strict_unknown_fields
cargo run --example path_policy_sandbox
cargo run --example witness_and_q_config
cargo run --example ocel_lifecycle_export
```
