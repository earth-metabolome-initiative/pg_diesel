# `PostgreSQL` Diesel Models and Schemas

[![CI](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Rust%20CI/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![Security Audit](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Security%20Audit/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Codecov](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel/branch/main/graph/badge.svg)](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel)

This library provides an easy way to analyze and work with `PostgreSQL` metadata, making it useful for metaprogramming tasks such as code generation based on database schemas.

Diesel models and schemas for `PostgreSQL` system catalogs (`pg_catalog`, `information_schema`) and `PostGIS` (`public`).

## What it does

- Provides Diesel schemas for **216** `PostgreSQL` metadata tables and views across:
  - `information_schema`: Standard SQL metadata, including 64 tables and views
  - `pg_catalog`: `PostgreSQL` system catalogs, including 149 tables and views
  - `public`: `PostGIS` geometry/geography columns
- Includes models for querying system catalogs type-safely
- Offers `PgDieselDatabase` for runtime database introspection
- Implements `sql_traits` for generic metadata access

## Features

- **serde**: Enables `Serialize`/`Deserialize` for all models (optional)
- **routines**: Includes the `routines` table with 88 columns (requires Diesel's `128-column-tables` feature), hence much longer compile times
- **postgres-14**: Support for `PostgreSQL` 14-specific schema features
- **postgres-15**: Support for `PostgreSQL` 15-specific schema features  
- **postgres-16**: Support for `PostgreSQL` 16-specific schema features
- **postgres-17**: Support for `PostgreSQL` 17-specific schema features
- **postgres-18**: Support for `PostgreSQL` 18-specific schema features (default)

### `PostgreSQL` Version Compatibility

The library provides comprehensive support for `PostgreSQL` versions 14 through 18. Each version has its own feature flag that controls version-specific schema elements:

- Columns that don't exist in older versions are feature-gated appropriately
- Columns with different types or constraints across versions are handled correctly  
- Tables with different schemas between versions (like `pg_statistic_ext_data`) have version-specific definitions
- The CI system tests all supported `PostgreSQL` versions to ensure compatibility

By default, `postgres-18` is enabled. To use with an older `PostgreSQL` version, disable the default features and enable the appropriate version:

```toml
[dependencies]
pg_diesel = { version = "*", default-features = false, features = ["postgres-16"] }
```

## Known limitations

Some `PostgreSQL` types are excluded because Diesel can't map them:

- `anyarray` - polymorphic pseudo-type
- `pg_ndistinct`, `pg_dependencies`, `pg_mcv_list`, `_pg_statistic` - internal statistics types

Columns with these types are omitted from generated schemas.

Additionally, Diesel requires tables that need to appear in the same query to be marked with the `allow_tables_to_appear_in_same_query!` macro. While we have added this macro for a selection of the most commonly queried tables, not all combinations are covered as we do not know all possible use cases. Feel free to send a pull request if you need additional combinations.

**`PostgreSQL` Version Support**: The library now provides comprehensive support for `PostgreSQL` versions 14 through 18. Schema differences between versions are handled through feature flags, with extensive testing across all supported versions. Each version-specific feature flag ensures that only appropriate columns and tables are included for that `PostgreSQL` version.

**Large Tables**: The [`routines`](https://www.postgresql.org/docs/current/infoschema-routines.html) table is excluded by default because it has 88 columns, requiring Diesel's `128-column-tables` feature, which significantly increases compile times. Enable it via the `routines` feature flag. Several important tables like [`columns`](https://www.postgresql.org/docs/current/infoschema-columns.html) have more than 32 columns and require Diesel's `64-column-tables` feature, which is enabled by default.

**Compile Times**: With the default features (including `postgres-18` but excluding `routines`), the crate compiles in about 60 seconds. Enabling the `routines` feature increases compile times to about 170 seconds on a typical development machine. The comprehensive CI system tests all `PostgreSQL` versions and feature combinations to ensure reliability across the supported range.
