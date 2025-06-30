# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), that adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Numbers in all JSON files are now formatted with up to 4 decimal places
- Support for top-level arrays in JSON and YAML files

### Fixed

- CLI can now be updated on Windows on ARM CPUs
- Argon can now run on older Linux versions (with `glibc 2.35+`)
- Updating instance meta now triggers client sync

### Changed

- Scripts are now written to the filesystem even if they are empty (syncback)
- Sourcemap regeneration is now only triggered by relevant changes
- Project details are now only synced when relevant project properties change
- `argon config -l` now displays only modified settings in `Current` column

## [2.0.24] - 2025-04-28

### Added

- Support for `Enum` attributes and `Content` properties
- Improved performance of `.rbxm` parsing
- `max_unsynced_changes` setting can now be set to much higher values
- Automatic Wally detection now supports `ServerPackages` and `DevPackages`
- Argon legacy namespace migration warning (`.src` to `init`) for every path that contains `.src` in its name

### Fixed

- `BasePart.Color` property is now properly converted between `Color3` and `Color3uint8` types
- Mismatched line endings no longer cause script diffs (all files are now normalized to `LF`)
- `--license` argument is now independent from the `--docs` argument in `argon init` command
- Empty StringValues (`.txt` files) are no longer ignored when syncing back
- Syncback now respects original file extensions so it no longer changes `.lua` to `.luau` when writing
- Package projects (that contain only root `$path`) are now properly synced back

### Changed

- Updated `rbx-dom` library to the latest major version
- Removed `line_ending` setting in favor of new `ignore_line_endings` setting
- All project templates now include improved `.gitignore`, `wally.toml` and `project.json` files
- Empty projects are no longer included in the tree (in result root project cannot be empty!)
- Argon legacy namespace for child file definitions (`.src`) is no longer supported by following middleware: `StringValue`, `RichStringValue`, `LocalizationTable`, `JsonModule`, `TomlModule`, `YamlModule`, `MsgpackModule`, `JsonModel`, `RbxmModel`, `RbxmxModel`

## [2.0.23] - 2025-02-05

### Fixed

- Legacy `.src` and `.data` files are working again when `rojo_mode` setting is enabled

## [2.0.22] - 2025-01-26

### Added

- Support for `.md` (Markdown) files that get transformed into `StringValue` containing rich text
- Separate `keep_duplicates` setting which was previously controlled by `rename_instances` setting

### Fixed

- Fix meta changes not being updated in the tree (sourcemap regeneration issue for new `.src` and `init` files)
- Argon no longer crashes when removing files that are described by multiple sync rules

### Changed

- `rojo_mode` setting is now `true` by default (will be removed in the future along with Argon legacy namespace)

## [2.0.21] - 2024-11-22

### Added

- `--force` parameter for `update` command that forces update even if there is no newer version

### Fixed

- Replaced old `argon run` with `argon serve` command in `place` template README
- Latest instance name is now saved in the tree when it was automatically renamed due to the forbidden characters

### Changed

- `name` field in project file is now optional (defaults to `default`)

## [2.0.20] - 2024-10-24

### Added

- Warning about unsynced changes when running `serve` and no client is connected (`max_unsynced_changes` setting)
- `changes_threshold` setting to control how many changes are allowed before prompting user for confirmation

### Fixed

- Sourcemap now regenerates when writing client changes

### Changed

- Removed `--disable` script flag - create data file with `Disabled` property set to `true` instead
- Syncback `ignoreGlobs` now match directories with `/**` suffix (not only its contents)

## [2.0.19] - 2024-09-19

### Added

- Support for automatic instance hydration with client tree
- New `empty` template that describes empty `DataModel` and contains only necessary files to get started
- New `selene` setting that allows to setup selene for codebase linting when initializing a new project

### Fixed

- Syncback instance name filter now works for more complex cases (e.g. `CON../`)

## [2.0.18] - 2024-09-08

### Added

- Automatic renaming instances with corrupted names (`rename_instances` setting)
- Support for instances with the same names (`rename_instances` setting)
- Support for syncing back `RunContext` property with `legacyScripts` disabled
- Pretty-printed project serialization when syncing back from client
- `line_ending` setting to control line endings when writing files

### Fixed

- `build` and `sourcemap` commands now properly read `--output` option with combination of `--async`
- Same level `.data.json` files for non-`Folder` instances can now parse properties correctly
- If project has `legacyScripts` disabled, scripts are now properly written when syncing back from client
- If the project path does not exist warn the user instead of failing

### Changed

- `include_docs` setting is now disabled by default

## [2.0.17] - 2024-08-21

### Fixed

- Child projects no longer cause root project to ignore file changes
- Attributes no longer serialize ambiguously for complex types

## [2.0.16] - 2024-08-18

### Added

- Support for workspace-defined Argon config (`argon.toml`)
- Default templates can now be updated when available (`update_templates` setting)
- Improved property parsing error details - filesystem and JSON path

### Changed

- `update` command now uses `cli`, `plugin`, `templates` or `all` argument instead of respective option

### Fixed

- Automatic updates are no longer cause output mess when running `update` command
- Plugin no longer updates when running `argon update` for the first time

## [2.0.15] - 2024-08-13

### Added

- `smart_paths` setting that makes specifying paths faster and easier
- Optional paths that can be specified in projects and are not required to exist

### Changed

- `RunContext` can no longer be specified inside script's source using comments
- Argon now returns proper exit code when it fails

### Fixed

- Wally package detection no longer requires `use_wally` setting when `detect_project` is enabled

## [2.0.14] - 2024-08-09

### Added

- Integration for `wally install` command when `use_wally` or `detect_project` setting is enabled
- Improved logging for client-server communication
- All properties can be now specified implicitly

### Fixed

- Moved `Packages` from ServerScriptService to ReplicatedStorage in `place` template

## [2.0.13] - 2024-07-19

### Added

- Support for `YAML` format that transforms to `ModuleScript` (both `.yaml` and `.yml` files extensions are allowed)
- Option to re-init existing project with missing template files

### Fixed

- Generated `wally.toml` package name no longer includes uppercase letters even if project or user name does

## [2.0.12] - 2024-07-11

### Fixed

- `Failed to clear temporary mesh models` error no longer appears after Roblox Studio update

## [2.0.11] - 2024-07-11

### Added

- Experimental support for syncing MeshPart's MeshId
- Argon now provides the link to [argon.wiki/changelog](https://argon.wiki/changelog) when a new update gets installed
- All project templates now include `Packages` folder in `use_wally` setting is enabled ([#71](https://github.com/argon-rbx/argon/issues/71))

### Changed

- Empty files like `.json`, `.csv` or `.msgpack` no longe cause errors

## [2.0.10] - 2024-07-05

### Added

- `--async` parameter is now user-exposed for `serve`, `build` and `sourcemap` commands ([#66](https://github.com/argon-rbx/argon/issues/66))
- `--default` parameter for `config` command that restores all settings to default values

### Fixed

- Newline character not being added to the Lua file header in some cases ([#62](https://github.com/argon-rbx/argon/pull/62))
- `serve` command now works as expected when running with `run_async` setting enabled

## [2.0.9] - 2024-06-25

### Added

- `package_manager` setting that allows to change package manager used when running commands with roblox-ts ([#58](https://github.com/argon-rbx/argon/issues/58))

### Fixed

- `argon init` now works properly with `PATH` argument and `roblox-ts`, `--yes` options ([#51](https://github.com/argon-rbx/argon/issues/51))

## [2.0.8] - 2024-06-16

### Added

- `lua_extension` global setting to control file extension when writing scripts

### Changed

- `filePaths` in sourcemap are now relative instead of absolute

### Fixed

- Sourcemap now includes project files in `filePaths`
- Script `Enabled` and `RunContext` flags no longer comment first line ([#28](https://github.com/argon-rbx/argon/issues/28))
- `rojo_mode` setting is now respected in two-way sync ([#47](https://github.com/argon-rbx/argon/issues/47))

## [2.0.7] - 2024-05-12

### Changed

- Arrays in `.data.json` and `*.project.json` files are now single-line by default

## [2.0.6] - 2024-05-08

### Changed

- `exec` command now looks for the first session with address instead of failing
- Floats are now saved in pretty-print format when two-way syncing properties

## [2.0.5] - 2024-05-06

### Fixed

- Instances with whitespace characters are now now synced back properly
- Instances with corrupted names now log the proper error message

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

[unreleased]: https://github.com/argon-rbx/argon/compare/2.0.24...HEAD
[2.0.24]: https://github.com/argon-rbx/argon/compare/2.0.23...2.0.24
[2.0.23]: https://github.com/argon-rbx/argon/compare/2.0.22...2.0.23
[2.0.22]: https://github.com/argon-rbx/argon/compare/2.0.21...2.0.22
[2.0.21]: https://github.com/argon-rbx/argon/compare/2.0.20...2.0.21
[2.0.20]: https://github.com/argon-rbx/argon/compare/2.0.19...2.0.20
[2.0.19]: https://github.com/argon-rbx/argon/compare/2.0.18...2.0.19
[2.0.18]: https://github.com/argon-rbx/argon/compare/2.0.17...2.0.18
[2.0.17]: https://github.com/argon-rbx/argon/compare/2.0.16...2.0.17
[2.0.16]: https://github.com/argon-rbx/argon/compare/2.0.15...2.0.16
[2.0.15]: https://github.com/argon-rbx/argon/compare/2.0.14...2.0.15
[2.0.14]: https://github.com/argon-rbx/argon/compare/2.0.13...2.0.14
[2.0.13]: https://github.com/argon-rbx/argon/compare/2.0.12...2.0.13
[2.0.12]: https://github.com/argon-rbx/argon/compare/2.0.11...2.0.12
[2.0.11]: https://github.com/argon-rbx/argon/compare/2.0.10...2.0.11
[2.0.10]: https://github.com/argon-rbx/argon/compare/2.0.9...2.0.10
[2.0.9]: https://github.com/argon-rbx/argon/compare/2.0.8...2.0.9
[2.0.8]: https://github.com/argon-rbx/argon/compare/2.0.7...2.0.8
[2.0.7]: https://github.com/argon-rbx/argon/compare/2.0.6...2.0.7
[2.0.6]: https://github.com/argon-rbx/argon/compare/2.0.5...2.0.6
[2.0.5]: https://github.com/argon-rbx/argon/compare/2.0.4...2.0.5
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
