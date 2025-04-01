# Changelog

All notable changes to this project will be documented in this file.

Versioning of this project adheres to the [Semantic Versioning](https://semver.org/spec/v2.0.0.html) spec.

## [0.2.2]

Released 2025-04-01

- Fixed a bug in the timestamp binary search logic that could cause the search to run forever instead of completing once the desired sequence number was found.

## [0.2.1]

Released 2025-03-30

- Removed a `dbg!()` log statement that was accidentally committed

## [0.2.0]

Released 2025-03-28

- Added `changeset` command to retrieve OSM changeset metadata or diffs
- Added `--watch` option to `replication` command (to run forever and poll for new replication files)
- Added seqno and timestamp to `replication` command output (use `--urls-only` for old behavior)
- Fix `replication` command to support changesets files (needed to handle off-by-one filenames)
- Updated README with install instructions and example usage
- Upgrade to Rust 2024
- Upgrade to ureq 3.0

## [0.1.0]

Released 2024-11-24

Initial release.
- Supports `node`, `way`, and `relation` subcommands for fetching info about OSM elements by ID.
- Supports `replication` subcommand for listing available replication files since a given timestamp or seqno.

[0.2.2]: https://github.com/jake-low/osm-cli/releases/tag/v0.2.2
[0.2.1]: https://github.com/jake-low/osm-cli/releases/tag/v0.2.1
[0.2.0]: https://github.com/jake-low/osm-cli/releases/tag/v0.2.0
[0.1.0]: https://github.com/jake-low/osm-cli/releases/tag/v0.1.0
