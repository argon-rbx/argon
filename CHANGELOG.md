# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), that adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Improved

- Instance source tracking and meta management

### Fixed

- `.src` and `init` files

## [2.0.0-pre5] - 2024-03-22

### Improved

- `plugin` subcommand now creates directory if the provided one does not exist
- Argon plugin gets installed automatically at the first Argon launch
- Config is now only read once

## [2.0.0-pre4] - 2024-03-21

### Added

- `plugin` subcommand that installs Argon plugin locally
- Argon CLI and plugin updater
- More customization with global config

### Changed

- `run` subcommand is now `serve`
- Changed default project name from `.argon.project.json` to `default.project.json`

### Fixed

- Sync rules no longer ignore specified project path, reported by [@Arid](https://github.com/AridAjd) and [@EthanMichalicek](https://github.com/EthanMichalicek) in [#23](https://github.com/argon-rbx/argon/issues/23)

## [2.0.0-pre3] - 2024-03-19

### Changed

- `run_async` option is now disabled by default

### Improved

- Free port searching speed
- Subcommand descriptions

### Fixed

- Path canonicalization on Windows
- Session management on Windows
- Crash reporting on Windows
- Release workflow

## [2.0.0-pre2] - 2024-03-18

### Fixed

- Argon installer not working properly with GitHub Actions

## [2.0.0-pre1] - 2024-03-18

### Added

- Brand new Argon CLI, written in Rust

[Unreleased]: https://github.com/argon-rbx/argon/compare/2.0.0-pre5...HEAD
[2.0.0-pre5]: https://github.com/argon-rbx/argon/compare/2.0.0-pre4...2.0.0-pre5
[2.0.0-pre4]: https://github.com/argon-rbx/argon/compare/2.0.0-pre3...2.0.0-pre4
[2.0.0-pre3]: https://github.com/argon-rbx/argon/compare/2.0.0-pre2...2.0.0-pre3
[2.0.0-pre2]: https://github.com/argon-rbx/argon/compare/2.0.0-pre1...2.0.0-pre2
[2.0.0-pre1]: https://github.com/argon-rbx/argon/compare/3057ca895492519fc29e7ab0bd8bdebc86d3e53c...2.0.0-pre1
