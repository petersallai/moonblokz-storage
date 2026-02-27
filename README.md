# moonblokz-storage

MoonBlokz storage contract crate for embedded `no_std` environments.

## Integration and Distribution

Current recommended integration model is Git dependency. Future crates.io
release model is documented below for later phase adoption.

### Current: Git Dependencies

`moonblokz-storage`:

```toml
[dependencies]
moonblokz-storage = { git = "https://github.com/petersallai/moonblokz-storage", default-features = false, features = ["backend-memory"] }
```

`moonblokz-chain-types`:

```toml
[dependencies]
moonblokz-chain-types = { git = "https://github.com/petersallai/moonblokz-chain-types" }
```

### Future: crates.io Dependencies

After crates.io publication, dependency wiring should switch to versioned crates:

```toml
[dependencies]
moonblokz-storage = { version = "0.1", default-features = false, features = ["backend-memory"] }
moonblokz-chain-types = "0.1"
```

Release expectations for crates.io phase:
- Keep backend feature exclusivity behavior unchanged.
- Keep `no_std` compatibility unchanged.
- Publish semver-compatible updates with changelog notes for API/contract changes.

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

## `BackendIo` Error Codes

`StorageError::BackendIo { code }` uses the following code map:

- Runtime:
- `1`: memory backend save-path received an oversized block input.
- `2`: memory backend read-path block parse failed for stored slot bytes.
- `210`: RP2040 flash page read failed.
- `211`: RP2040 flash page erase failed.
- `212`: RP2040 flash page write failed.
- `213`: RP2040 save path reached an unexpected backend branch.
- `220`: RP2040 flash page read failed during retrieve path.
- Test-only (`backend-rp2040` unit tests with mock flash):
- `230`: mock flash read out of bounds.
- `231`: mock flash erase range invalid/out of bounds.
- `232`: mock flash write out of bounds.

## Memory Backend Capacity Rule

For `backend-memory`, `STORAGE_SIZE` is interpreted as total storage bytes.

- Effective slot count is `STORAGE_SIZE / MAX_BLOCK_SIZE` (integer division).
- Any remainder bytes (`STORAGE_SIZE % MAX_BLOCK_SIZE`) are intentionally unused.
- Empty slot is identified by first byte `0` (version byte `0` means empty).
