//! Trait implementations connecting models to the `sql_traits` ecosystem.
//!
//! This module implements various traits from the `sql_traits` crate for the
//! model structs defined in this crate. These implementations enable generic
//! database introspection and code generation.
//!
//! ## Implemented Traits
//!
//! - `TableLike`: Implemented for
//!   [`Table`](crate::models::Table)
//! - `ColumnLike`: Implemented for
//!   [`Column`](crate::models::Column)
//! - `CheckConstraintLike`: Implemented for
//!   [`CheckConstraint`](crate::models::CheckConstraint)
//! - `ForeignKeyLike`: Implemented for
//!   [`KeyColumnUsage`](crate::models::KeyColumnUsage)
//! - `FunctionLike`: Implemented for [`PgProc`](crate::models::PgProc)
//! - `UniqueIndexLike`: Implemented for [`PgIndex`](crate::models::PgIndex)
//! - `SchemaLike`: Implemented for [`Schemata`](crate::models::Schemata)
//! - [`HasOid`](crate::traits::HasOid): Implemented for various catalog types
//! - [`PostgresType`](crate::traits::PostgresType): Implemented for types that
//!   resolve their Postgres type

mod check_constraint_like;
mod column_grant_like;
mod column_like;
mod foreign_key_like;
mod function_like;
mod index_like;
mod oid;
mod policy_like;
mod postgres_type;
mod role_like;
mod schema_like;
mod table_grant_like;
mod table_like;
mod trigger_like;

pub use column_grant_like::RoleColumnGrantsMetadata;
pub use table_grant_like::{RoleTableGrantsMetadata, string_to_action};
