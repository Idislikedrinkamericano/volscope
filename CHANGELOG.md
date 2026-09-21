# Changelog

All notable changes to VolScope are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and releases use semantic versioning after the public API stabilizes.

## [Unreleased]

### Added

- Read-only Research Copilot V0 with replay/live snapshot binding.
- Structured evidence references for volatility, dealer exposure, and data-quality answers.
- Deterministic fallback responses that work without an external model credential.

## [0.1.0] - 2026-08-13

### Added

- Independent VolScope repository structure.
- Rust replay/live analytics server and React workstation.
- Chinese beginner guide and decision-support walkthrough.
- Chinese-first project README with a detailed capability map, operating
  workflow, local API reference, and maintainer contact.
- Reproducible local and container startup paths.
- Open-source governance, security, contribution, CI, and release standards.

### Security

- Loopback-only default binding.
- Process-memory-only Longbridge credential handling.
- Server-disabled paper execution with independent account, freshness, and
  typed-confirmation gates.
- Local Longbridge OAuth compatibility patch upgrades the transitive TLS stack
  past RUSTSEC-2026-0098, RUSTSEC-2026-0099, and RUSTSEC-2026-0104.
- RustSec CI now guards the reviewed lockfile-only `RUSTSEC-2026-0235`
  exception with an all-target dependency-reachability check.
- Updated the transitive `nanoid` dependency past `GHSA-2v37-7h3g-55p8`.
- Publication-time private-path, market-data, oversized-file, and secret scans.

Initial public preview. No compatibility guarantee is provided before 1.0.
