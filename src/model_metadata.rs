//! Metadata wrappers providing enriched information about database objects.
//!
//! This module provides metadata types that wrap the basic model structs with
//! additional contextual information loaded from related tables:
//!
//! - [`TableMetadata`]: Wraps a table with its columns, constraints, indices,
//!   and description
//! - [`ColumnMetadata`]: Wraps a column with its table, type information, and
//!   description
//!
//! These metadata types are used by the
//! [`PgDieselDatabase`](crate::database::PgDieselDatabase) to provide rich introspection
//! capabilities through the `sql_traits` trait system.

mod table_metadata;
pub use table_metadata::TableMetadata;
mod column_metadata;
pub use column_metadata::ColumnMetadata;
mod trigger_metadata;
pub use trigger_metadata::TriggerMetadata;
mod policy_metadata;
pub use policy_metadata::PolicyMetadata;
