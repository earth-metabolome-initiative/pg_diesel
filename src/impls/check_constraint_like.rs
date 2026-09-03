//! Implementation of [`CheckConstraintLike`] for [`CheckConstraint`].

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::metadata::CheckMetadata,
    traits::{CheckConstraintLike, DatabaseLike, Metadata},
};

use crate::{PgDieselDatabase, models::CheckConstraint};

impl Metadata for CheckConstraint {
    type Meta = CheckMetadata<Self>;
}

/// Returns the metadata `database` holds for `constraint`.
fn metadata<'db>(
    constraint: &CheckConstraint,
    database: &'db PgDieselDatabase,
) -> Result<&'db CheckMetadata<CheckConstraint>, LookupError> {
    database
        .check_constraint_metadata(constraint)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::CheckConstraint,
            object: constraint.constraint_name.clone(),
        })
}

impl CheckConstraintLike for CheckConstraint {
    type DB = PgDieselDatabase;

    fn expression<'db>(&'db self, database: &'db Self::DB) -> &'db sqlparser::ast::Expr {
        database
            .check_constraint_metadata(self)
            .expect("Check constraint must exist in database")
            .expression()
    }

    fn table<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<&'db <Self::DB as DatabaseLike>::Table, LookupError> {
        Ok(metadata(self, database)?.table())
    }

    fn columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError> {
        Ok(metadata(self, database)?.columns())
    }

    fn functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + 'db, LookupError>
    {
        Ok(metadata(self, database)?.functions())
    }
}
