# Changelog

All notable changes to Aevum will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Aevum time-travel debugging platform
- Rust trace engine with vector clocks and zstd compression
- Python agent with sys.settrace instrumentation
- Go agent with goroutine tracking
- Node.js agent with Inspector Protocol
- C/C++ agent stub with LLVM hooks
- JVM agent with ASM bytecode instrumentation
- Go distributed coordinator with trace merging
- Race condition detection via causal graph analysis
- TypeScript/React UI with D3 timeline visualization
- Cytoscape.js causality graph visualization
- Interactive playback controls
- CLI with attach, record, replay, rewind, inspect commands
- Docker Compose deployment
- GitHub Actions CI/CD pipeline
- Protocol Buffers definitions
- JSON schemas for trace files and configuration
- Comprehensive documentation

### Technical Details
- Append-only trace log with atomic writes
- Snapshot + delta compression for memory efficiency
- Clock skew compensation for distributed traces
- Deterministic scheduler for replay
- 15 event types captured

## [0.1.0] - 2026-01-13

### Added
- Initial project structure
- Core architecture design
- Basic functionality across all components

---

## Categories

- `Added` for new features
- `Changed` for changes in existing functionality
- `Deprecated` for soon-to-be removed features
- `Removed` for now removed features
- `Fixed` for any bug fixes
- `Security` in case of vulnerabilities
