//! Core traits for `PostgreSQL` metadata types.

pub mod oid;
pub use oid::HasOid;
pub mod postgres_type;
pub use postgres_type::PostgresType;
