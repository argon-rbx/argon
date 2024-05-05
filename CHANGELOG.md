# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), that adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.4] - 2024-05-05

### Added

- File name verification to avoid creating corrupted instances (blocks some characters and names)

### Fixed

- `debug` command no longer errors even when succeeding on Windows
- `exec` command now actually focuses Roblox Studio when enabled on Windows

## [2.0.3] - 2024-05-04

### Added

- Support for values in boolean flags for `init` command, example: `--git=false`
- New setting `with_sourcemap` - always run commands with sourcemap generation
- New setting `build_xml` - build using XML format by default

### Changed

- You can now specify to update CLI or plugin only in `update` command
- Properties are now serialized alphabetically ([#25](https://github.com/argon-rbx/argon/pull/25))
- Renamed `auto_detect` setting to `detect_project`

## [2.0.2] - 2024-05-03

### Added

- Support for MessagePack (`.msgpack`) - binary format, great for storing big amount of data

### Changed

- Argon now uses the `.luau` extension by default when syncing back from Roblox Studio
- When running `argon plugin install` with no internet connection the bundled binary will be used

## [2.0.1] - 2024-05-02

### Fixed

- `ArgonEmpty` property is no longer serialized on child instances
- `math.huge` is no longer saved as JSON `null` (temporarily it's just a big number)

### Changed

- Increased client write request payload size limit from `256 KiB` to `512 MiB`!
- Error tracing when Argon fails to snapshot nested file or directory
- Significantly decreased initial file system snapshotting time (caused by Notify)

## [2.0.0] - 2024-05-01

### Added

- Full two-way sync
- Support for `legacyScripts` and `keepUnknowns` fields
- Ability to re-release the same version when needed
- Virtual file system for testing
- `plugin` command now fallbacks to bundled binary if user has no internet connection
- `update` command that updates both CLI and plugin regardless of global configuration
- Community stats tracking
- Helper scripts

### Changed

- Instance source tracking and meta management
- Standard file system with additional methods
- Argon network protocol now uses MessagePack instead of JSON
- Sessions that crashed now get removed from `sessions.toml` file

### Fixed

- `.src` and `init` files in sourcemap generation
- `Open In Editor` now opens folders only if instance has no other sources
- Plugin now installs and updates correctly on Windows

## [2.0.0-pre5] - 2024-03-22

### Changed

- `plugin` command now creates directory if the provided one does not exist
- Argon plugin gets installed automatically at the first Argon launch
- Config is now only read once

## [2.0.0-pre4] - 2024-03-21

### Added

- `plugin` command that installs Argon plugin locally
- Argon CLI and plugin updater
- More customization with global config

### Changed

- `run` command is now `serve`
- Changed default project name from `.argon.project.json` to `default.project.json`

### Fixed

- Sync rules no longer ignore specified project path ([#23](https://github.com/argon-rbx/argon/issues/23))

## [2.0.0-pre3] - 2024-03-19

### Changed

- `run_async` option is now disabled by default

### Changed

- Free port searching speed
- Command descriptions

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

[unreleased]: https://github.com/argon-rbx/argon/compare/2.0.4...HEAD
[2.0.4]: https://github.com/argon-rbx/argon/compare/2.0.3...2.0.4
[2.0.3]: https://github.com/argon-rbx/argon/compare/2.0.2...2.0.3
[2.0.2]: https://github.com/argon-rbx/argon/compare/2.0.1...2.0.2
[2.0.1]: https://github.com/argon-rbx/argon/compare/2.0.0...2.0.1
[2.0.0]: https://github.com/argon-rbx/argon/compare/2.0.0-pre5...2.0.0
[2.0.0-pre5]: https://github.com/argon-rbx/argon/compare/2.0.0-pre4...2.0.0-pre5
[2.0.0-pre4]: https://github.com/argon-rbx/argon/compare/2.0.0-pre3...2.0.0-pre4
[2.0.0-pre3]: https://github.com/argon-rbx/argon/compare/2.0.0-pre2...2.0.0-pre3
[2.0.0-pre2]: https://github.com/argon-rbx/argon/compare/2.0.0-pre1...2.0.0-pre2
[2.0.0-pre1]: https://github.com/argon-rbx/argon/compare/1.3.0...2.0.0-pre1
