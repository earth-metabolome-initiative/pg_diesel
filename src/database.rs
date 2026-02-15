//! Submodule providing the `PgDieselDatabase` struct which holds data queried from
//! the `PostgreSQL` information schema and implements the
//! `DatabaseLike` trait.

use sql_traits::structs::GenericDB;

use crate::model_metadata::TriggerMetadata;
use crate::models::{
    CheckConstraint, Column, KeyColumnUsage, PgIndex, PgPolicyTable, PgProc, PgRole,
    RoleColumnGrants, RoleTableGrants, Schemata, Table,
};
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
/// - [`Schemata`] from `information_schema.schemata` as the schema type
/// - [`RoleTableGrants`] from `information_schema.role_table_grants` as the table
///   grant type
/// - [`RoleColumnGrants`] from `information_schema.role_column_grants` as the
///   column grant type
///
/// The `PgDieselDatabase` implements
/// `DatabaseLike`, providing methods to
/// iterate over tables, columns, foreign keys, and other database objects.
pub type PgDieselDatabase = GenericDB<
    Table,
    Column,
    PgIndex,
    PgIndex,
    KeyColumnUsage,
    PgProc,
    CheckConstraint,
    TriggerMetadata,
    PgPolicyTable,
    PgRole,
    Schemata,
    RoleTableGrants,
    RoleColumnGrants,
>;
