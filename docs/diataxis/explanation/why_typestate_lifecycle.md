# Why Typestate Lifecycle?

This explanation explains the architectural decision behind compile-time typestate check wrappers in star-toml.

## The Problem: Loose Configuration State

In standard Rust applications, loaded configuration is often represented as a plain struct. While simple, this does not track whether values have been merged, validated, or approved by the security boundary. An application might access configuration before validation is complete, leading to runtime failures or security vulnerabilities.

## The Solution: Typestate Lifecycles

Star-toml introduces strict compile-time typestate patterns to separate stages:
- **Raw**: Unstructured, unvalidated parsing state.
- **Merged**: Multiple files overlaid with environment variables resolved.
- **Validated**: Explicitly verified against rules, immutable.
- **Frozen**: Locked for cryptographical signature calculation.
- **Admitted**: Wrapped inside an envelope proving authenticity.

By hiding fields and restricting structures, applications cannot compile unless the code executes the complete validation sequence.
