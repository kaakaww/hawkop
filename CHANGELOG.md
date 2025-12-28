# Changelog

All notable changes to HawkOp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2025-12-27

### Added

- `hawkop audit list` command with filtering by activity type, user, email, and date ranges
- `hawkop oas list` command to list hosted OpenAPI specifications
- `hawkop config list` command to list scan configurations
- `hawkop secret list` command to list user secret names
- `hawkop completion` command for shell completions (bash, zsh, fish, powershell)
- `--type` filter for `hawkop app list` to filter by cloud/standard applications
- Comprehensive unit tests for error types, display models, and output formatters

### Changed

- Refactored date parsing to use let-chains for cleaner conditionals

## [0.1.0] - 2025-12-27

### Added

- Initial release
- `hawkop init` for interactive setup
- `hawkop status` to display configuration status
- `hawkop org list|set|get` for organization management
- `hawkop app list` with pagination and parallel fetching
- `hawkop scan list` with filtering by status, environment, and application
- `hawkop user list` to list organization members
- `hawkop team list` to list organization teams
- `hawkop policy list` to list scan policies
- `hawkop repo list` to list repositories in attack surface
- Table and JSON output formats (`--format table|json`)
- Global `--org` flag for organization override
- Global `--debug` flag for verbose logging
- Reactive per-endpoint rate limiting
- Parallel pagination for large datasets

[Unreleased]: https://github.com/kaakaww/hawkop/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.2
[0.2.1]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.1
[0.2.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kaakaww/hawkop/releases/tag/v0.1.0
