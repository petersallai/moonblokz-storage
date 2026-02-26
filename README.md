# moonblokz-storage

MoonBlokz storage contract crate for embedded `no_std` environments.

## Backend Feature Selection

Exactly one backend feature must be enabled at compile time:

- `backend-memory`
- `backend-rp2040`

Default feature is `backend-memory`.

### Examples

```sh
# Default (backend-memory)
cargo check

# Explicit memory backend
cargo check --no-default-features --features backend-memory

# Explicit RP2040 backend
cargo check --no-default-features --features backend-rp2040
```

These combinations must fail:

```sh
# No backend selected
cargo check --no-default-features

# Multiple backends selected
cargo check --no-default-features --features "backend-memory backend-rp2040"
```
