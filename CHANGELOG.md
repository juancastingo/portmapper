# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-06

### Added
- Initial public release of `portmapper`.
- Local listening socket discovery across TCP/UDP and IPv4/IPv6 via Linux `/proc/net`.
- Inode resolution mapping ports to PID and process command name.
- UID resolution mapping socket owners to local usernames.
- Snapshot creation and local persistence (`portmapper save`).
- Time-series snapshot diffing (`portmapper diff`) detecting:
  - Newly opened ports (`added`)
  - Closed ports (`removed`)
  - Process ownership changes / restarts (`changed`)
- Public exposure detection (`0.0.0.0` or `::`).
- CI integration support with `--fail-on-new` and `--json`.
