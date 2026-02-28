# Release Process

This document defines the release workflow for `moonblokz-storage`.

## Versioning Policy

- Use SemVer (`MAJOR.MINOR.PATCH`).
- `PATCH`: bug fixes and internal improvements without public API changes.
- `MINOR`: backward-compatible API additions and feature expansions.
- `MAJOR`: breaking API/behavior changes.

## Changelog Policy

- Keep `CHANGELOG.md` updated using Keep a Changelog style sections.
- Record only user-visible changes:
- Added
- Changed
- Fixed
- Removed

## Pre-Release Checklist

1. Ensure all intended changes are merged to `main`.
2. Update `Cargo.toml` version.
3. Update `CHANGELOG.md` for the target version/date.
4. Run local quality gates:
   - `./run_tests.sh`
5. Verify README dependency snippets and docs links.
6. Commit release-prep changes.

## Tagging and GitHub Release

1. Create annotated tag:
   - `git tag -a vX.Y.Z -m "moonblokz-storage vX.Y.Z"`
2. Push commit and tag:
   - `git push origin main`
   - `git push origin vX.Y.Z`
3. Create GitHub Release from tag and copy the matching changelog section.

## Post-Release

1. Bump to next development version only when needed.
2. Keep changelog ready with a new `Unreleased` section.
