# `PostgreSQL` Diesel Models and Schemas

[![CI](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Rust%20CI/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![Security Audit](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Security%20Audit/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Codecov](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel/branch/main/graph/badge.svg)](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel)
[![Crates.io](https://img.shields.io/crates/v/pg_diesel.svg)](https://crates.io/crates/pg_diesel)
[![Docs.rs](https://docs.rs/pg_diesel/badge.svg)](https://docs.rs/pg_diesel)

This library provides an easy way to analyze and work with `PostgreSQL` metadata, making it useful for metaprogramming tasks such as code generation based on database schemas.

Diesel models and schemas for `PostgreSQL` system catalogs (`pg_catalog`, `information_schema`) and `PostGIS` (`public`).

## What it does

- Provides Diesel schemas for **192** `PostgreSQL` metadata tables and views across:
  - `information_schema`: Standard SQL metadata
  - `pg_catalog`: `PostgreSQL` system catalogs
  - `public`: `PostGIS` geometry/geography columns
- Includes models for querying system catalogs type-safely
- Offers `PgDieselDatabaseBuilder` for runtime database introspection
- Implements `sql_traits` for generic metadata access

## Features

- **serde**: Enables `Serialize`/`Deserialize` for all models (optional)

## Known limitations

Some `PostgreSQL` types are excluded because Diesel can't map them:

- `anyarray` - polymorphic pseudo-type
- `pg_ndistinct`, `pg_dependencies`, `pg_mcv_list`, `_pg_statistic` - internal statistics types

Columns with these types are omitted from generated schemas.
