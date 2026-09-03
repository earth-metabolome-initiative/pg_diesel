//! Trait implementations connecting models to the `sql_traits` ecosystem.

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
mod view_like;

pub use column_grant_like::RoleColumnGrantsMetadata;
pub use table_grant_like::{RoleTableGrantsMetadata, string_to_action};
