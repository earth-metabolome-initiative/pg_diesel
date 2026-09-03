//! Implementation of [`IndexLike`] for [`PgIndexEntry`].

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::metadata::UniqueIndexMetadata,
    traits::{DatabaseLike, IndexLike, Metadata},
};
use sqlparser::ast::Expr;

use crate::{PgDieselDatabase, model_metadata::PgIndexEntry};

impl Metadata for PgIndexEntry {
    type Meta = UniqueIndexMetadata<Self>;
}

impl IndexLike for PgIndexEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> Option<&str> {
        Some(PgIndexEntry::name(self))
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn schema(&self) -> Option<&str> {
        Some(PgIndexEntry::schema(self))
    }

    fn schema_is_quoted(&self) -> bool {
        true
    }

    fn table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .index_metadata(self)
            .expect("Index must exist in database")
            .table()
    }

    fn expression<'db>(&'db self, database: &'db Self::DB) -> Result<&'db Expr, LookupError>
    where
        Self: 'db,
    {
        Ok(database
            .index_metadata(self)
            .ok_or_else(|| LookupError::ObjectNotInDatabase {
                object_kind: ObjectKind::Index,
                object: format!(
                    "{}.{}",
                    PgIndexEntry::schema(self),
                    PgIndexEntry::name(self)
                ),
            })?
            .expression())
    }
}
