//! Submodule providing the `PgDieselDatabase` struct which holds data queried from
//! the `PostgreSQL` information schema and implements the
//! `DatabaseLike` trait.

use sql_traits::structs::GenericDB;

use crate::model_metadata::TriggerMetadata;
use crate::models::{CheckConstraint, Column, KeyColumnUsage, PgIndex, PgProc, Table};
mod key_column_usage_metadata;
pub use key_column_usage_metadata::KeyColumnUsageMetadata;
mod pg_proc_metadata;
pub use pg_proc_metadata::PgProcMetadata;

mod builder;
pub use builder::{PgDatabaseBuildError, PgDieselDatabaseBuilder};

/// Type alias representing a `PostgreSQL` database with loaded metadata.
///
/// This is a specialization of [`GenericDB`] configured for `PostgreSQL`, using:
/// - [`Table`] from `information_schema.tables` as the
///   table type
/// - [`Column`] from `information_schema.columns` as the
///   column type
/// - [`PgIndex`] from `pg_catalog.pg_index` as the
///   unique index type
/// - [`KeyColumnUsage`] from
///   `information_schema.key_column_usage` as the foreign key type
/// - [`PgProc`] from `pg_catalog.pg_proc` as the
///   function type
/// - [`CheckConstraint`] from
///   `information_schema.check_constraints` as the check constraint type
/// - [`TriggerMetadata`] from `information_schema.triggers` as the trigger type
///
/// The `PgDieselDatabase` implements
/// `DatabaseLike`, providing methods to
/// iterate over tables, columns, foreign keys, and other database objects.
pub type PgDieselDatabase =
    GenericDB<Table, Column, PgIndex, KeyColumnUsage, PgProc, CheckConstraint, TriggerMetadata>;
