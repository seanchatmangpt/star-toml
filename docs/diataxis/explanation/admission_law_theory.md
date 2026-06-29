# Theory of the DfCM Admission Law

This article explains the theoretical and mathematical background governing the release admission loop in star-toml.

## Vision 2030 Core Principles

Under Vision 2030, configuration documentation and code standing are not independent narratives; they are components of a single, verifiable equation.

We express this using the Admission Standing formula:

$$q_{release} = 1 \iff q_{code} \land q_{docs} \land q_{api} \land q_{examples} \land q_{lsp} \land q_{publish} \land |F| = 0$$

- $F$: The failset of caveats/counterexamples (must be empty).
- $q_{code}$: 100% test coverage.
- $q_{docs}$: Documentation matches physical schemas.

Rather than managing caveats as prose warnings, they are implemented as executable counterexamples. This prevents release overclaims and ensures that the codebase status is mathematically proven.
