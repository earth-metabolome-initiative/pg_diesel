//! Implementation of [`IndexLike`] for [`PgIndex`].

use sql_traits::{
    structs::metadata::UniqueIndexMetadata,
    traits::{IndexLike, Metadata},
};
use sqlparser::ast::Expr;

use crate::{PgDieselDatabase, models::PgIndex};

impl IndexLike for PgIndex {
    type DB = PgDieselDatabase;

    fn table<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> &'db <Self::DB as sql_traits::traits::DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .index_metadata(self)
            .expect("Index must exist in database")
            .table()
    }

    fn expression<'db>(&'db self, database: &'db Self::DB) -> &'db Expr
    where
        Self: 'db,
    {
        database
            .index_metadata(self)
            .expect("Index must exist in database")
            .expression()
    }
}

impl Metadata for PgIndex {
    type Meta = UniqueIndexMetadata<Self>;
}
