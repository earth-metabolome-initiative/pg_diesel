//! Implementation of [`UniqueIndexLike`] for [`PgIndex`].
//!
//! This module implements the
//! [`UniqueIndexLike`] trait for the
//! [`PgIndex`] model from `pg_catalog.pg_index`, enabling generic introspection
//! of unique indexes and primary keys.
//!
//! The implementation provides access to:
//! - The table that the index is defined on
//! - The parsed index expression (from `pg_get_indexdef`)
//!
//! Metadata is provided by [`UniqueIndexMetadata`] from the `sql_traits` crate.

use sql_traits::{structs::metadata::UniqueIndexMetadata, traits::Metadata};

use crate::models::PgIndex;

impl Metadata for PgIndex {
    type Meta = UniqueIndexMetadata<Self>;
}
