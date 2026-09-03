//! Implementation of [`ColumnLike`] for [`Column`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::TargetName,
    traits::{ColumnCollation, ColumnLike, DatabaseLike, Metadata, NamedColumnCollation},
};

use crate::{PgDieselDatabase, model_metadata::ColumnMetadata, models::Column};

impl Metadata for Column {
    type Meta = ColumnMetadata;
}

/// Returns the metadata `database` holds for `column`.
fn metadata<'db>(
    column: &Column,
    database: &'db PgDieselDatabase,
) -> Result<&'db ColumnMetadata, LookupError> {
    database
        .column_metadata(column)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Column,
            object: format!(
                "{}.{}.{}",
                column.table_schema, column.table_name, column.column_name
            ),
        })
}

impl ColumnLike for Column {
    type DB = PgDieselDatabase;

    fn column_name(&self) -> &str {
        &self.column_name
    }

    fn column_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn column_doc<'db>(&'db self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?
            .description()
            .map(|desc| desc.description.as_str()))
    }

    fn table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .column_metadata(self)
            .expect("Column must exist in database")
            .table()
    }

    fn is_generated(&self) -> bool {
        self.is_generated == "ALWAYS"
            || self
                .column_default
                .as_ref()
                .is_some_and(|d| d.starts_with("nextval"))
            || self.is_identity.as_ref().is_some_and(|i| i == "YES")
    }

    fn data_type<'db>(&'db self, database: &'db Self::DB) -> Cow<'db, str> {
        metadata(self, database).map_or(Cow::Borrowed(""), |meta| {
            Cow::Borrowed(meta.pg_type().typname.as_str())
        })
    }

    fn collation<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<ColumnCollation<'db>, LookupError> {
        let Some(name) = self.collation_name.as_deref() else {
            // Empty collation columns mean the database default.
            return Ok(ColumnCollation::DatabaseDefault);
        };
        let mut target = TargetName::new(name, true);
        if let Some(schema) = self.collation_schema.as_deref() {
            target = target.with_schema(schema, true);
        }
        let deterministic = metadata(self, database)?.collation_is_deterministic();
        Ok(ColumnCollation::Named(
            NamedColumnCollation::new(target).with_postgres_deterministic(deterministic),
        ))
    }

    fn is_nullable(&self, _database: &Self::DB) -> Result<bool, LookupError> {
        Ok(self.__is_nullable == "YES")
    }

    fn default_value(&self) -> Option<String> {
        self.column_default.clone()
    }
}
