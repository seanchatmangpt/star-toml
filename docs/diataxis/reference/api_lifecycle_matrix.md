# API Lifecycle Matrix Reference

This reference details the compile-time typestate wrappers and transitions defining the star-toml loading pipeline.

## Typestate Structs

| Struct | Pipeline Phase | Mutability | Constructible By |
|---|---|---|---|
| `RawConfig` | Ingestion Phase | Read-Write | `TrustedLoader::load` |
| `MergedConfig` | Layering Phase | Read-Write | `RawConfig::merge` |
| `ValidatedConfig` | Validation Phase | Read-Only | `MergedConfig::validate` |
| `FrozenConfig` | Admission Phase | Read-Only | `ValidatedConfig::freeze` |
| `AdmittedConfig<T>` | Envelope Phase | Read-Only | `FrozenConfig::admit` |

## State Transitions

```
TrustedLoader::load() -> RawConfig
                        |
                        +---> .merge() -> MergedConfig
                                            |
                                            +---> .validate() -> ValidatedConfig
                                                                   |
                                                                   +---> .freeze() -> FrozenConfig
                                                                                        |
                                                                                        +---> .admit() -> AdmittedConfig<T>
```

A final `AdmittedConfig<T>` can only be obtained through this exact sequence.
