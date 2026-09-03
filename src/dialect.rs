//! The `PostgreSQL` dialect the introspected catalogs speak.

use sql_traits::traits::{ColumnLike, DialectLike, TypeMatch};

use crate::{PgDieselDatabase, models::Column};

/// The dialect of a database read through `pg_catalog`.
///
/// A catalog row records the type the server resolved, so a predicate never
/// answers [`TypeMatch::Maybe`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostgresDialect;

impl PostgresDialect {
    /// Whether the column's resolved catalog type is `type_name`.
    fn declares(database: &PgDieselDatabase, column: &Column, type_name: &str) -> TypeMatch {
        if column.data_type(database) == type_name {
            TypeMatch::Yes
        } else {
            TypeMatch::No
        }
    }
}

impl DialectLike for PostgresDialect {
    type DB = PgDieselDatabase;
    type Match = TypeMatch;

    fn is_bool(&self, database: &Self::DB, column: &Column) -> Self::Match {
        Self::declares(database, column, "bool")
    }

    fn is_uuid(&self, database: &Self::DB, column: &Column) -> Self::Match {
        Self::declares(database, column, "uuid")
    }
}
