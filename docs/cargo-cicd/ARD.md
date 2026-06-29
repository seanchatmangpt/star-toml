# cargo-cicd: Architecture Requirements Document (ARD)

## Architecture Diagram

```
Human
  │
Coding Agent
  │
MCP
  │
Cargo Commands
  │
cargo-cicd
──────────────────────────────── (Execution Border)
Admission | Planning | Task Graph | Verification | Receipts | Standing
────────────────────────────────
just (Outside-world actuator)
────────────────────────────────
Docker | Terraform | Kubernetes | Nuxt | React Native | Cloud | Filesystem | Git | Registries
────────────────────────────────
Evidence
  │
Verifier
  │
Standing
```

---

## Architectural Layers

### Layer 1: Operational Law
Defined via `cargo-cicd.toml` rules and validations.

### Layer 2: Planning
Calculates the workspace topology, deployment topology, task execution graph, and the DfCM matrix.

### Layer 3: Execution
Invokes Cargo commands, `just` files, and external system actuators.

### Layer 4: Verification
Verifies diagnostics, cryptographical receipts, file freshness, test coverage, and the threat matrix.

### Layer 5: Standing
Confers the final standing status:
* `ADMITTED`
* `REFUSED`
* `DEV_ONLY`
* `AUTH_BLOCKED`
* `REGISTRY_BLOCKED`
* `OUT_OF_SCOPE`

---

## Authority Model

* **Human**: Defines organizational policies and writes constraints.
* **Agent**: Performs bounded work (repair, optimization, generation).
* **just**: Interacts with the filesystem, networks, and cloud providers.
* **Verifier**: Assesses evidence and confers standing.
* **Receipts**: Provides cryptographic evidence proving standing.

---

## Core Invariants

1. Development maximizes freedom; production maximizes impossibility.
2. No production standing without verifier gates.
3. No verifier without receipts.
4. No receipt without evidence.
5. No evidence without execution.
6. No execution outside admitted Cargo commands.
7. No external tool, agent, or editor may confer standing.
8. No raw production configurations are ever allowed in memory.

---

## Security Model

* **Construction-First Security**: Systems are designed such that invalid states are unrepresentable.
* **Impossible-State Harvesting**: Active check gates turn misconfigurations into hard compilation or verification failures.
* **Portable Verification**: Verification targets can run identically in browser/WASM contexts as they do in native servers.
* **PQC-Ready Receipt Binding**: Verification tokens use cryptographic digests bound directly to input/output artifacts.

### Threat Classification Matrix

| Category | Description |
|---|---|
| **Impossible** | Forbidden by compiler typestates or verifier assertions. |
| **Improbable** | Hardened by cryptographical signatures. |
| **Bounded** | Restricted by sandbox or path boundary policies. |
| **Detected** | Caught by runtime diagnostics. |
| **Unhandled** | **Must not ship.** Cannot bypass verification. |

---

## Deployment Profiles

* Deployment profiles are executable examples.
* Examples provide operational coverage.
* Examples become deployment fixtures.
* Examples generate receipts.
* Deployment support is admitted only through verified examples.

---

## Chatman's Law

> Operational substrates reshape organizations.
> Organizations converge around admitted operational law.
> Communication follows deployment authority.
> Shared Cargo operational law becomes shared organizational language.
