//! Implementation of [`FunctionLike`] for [`PgFunction`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::TargetName,
    traits::{FunctionLike, Metadata},
};
use sqlparser::ast::{Expr, FunctionCalledOnNull, FunctionDefinitionSetParam, FunctionSecurity};

use crate::{PgDieselDatabase, database::PgProcMetadata, model_metadata::PgFunction};

impl Metadata for PgFunction {
    type Meta = PgProcMetadata;
}

/// Returns the metadata `database` holds for `function`.
fn metadata<'db>(
    function: &PgFunction,
    database: &'db PgDieselDatabase,
) -> Result<&'db PgProcMetadata, LookupError> {
    database
        .function_metadata(function)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Function,
            object: format!("{}.{}", function.schema(), function.model().proname),
        })
}

impl FunctionLike for PgFunction {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model().proname
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn target_name(&self) -> TargetName<'_> {
        TargetName::new(self.name(), true).with_schema(self.schema(), true)
    }

    fn body(&self) -> Option<&str> {
        Some(&self.model().prosrc)
    }

    fn body_expression(&self) -> Option<&Expr> {
        // `prosqlbody` holds a node tree, not SQL, so it never parses back.
        None
    }

    fn returns_set(&self) -> bool {
        self.model().proretset
    }

    fn language(&self) -> Option<&str> {
        PgFunction::language(self)
    }

    fn language_is_quoted(&self) -> bool {
        true
    }

    fn security_mode(&self) -> FunctionSecurity {
        if self.model().prosecdef {
            FunctionSecurity::Definer
        } else {
            FunctionSecurity::Invoker
        }
    }

    fn null_input_behavior(&self) -> FunctionCalledOnNull {
        if self.model().proisstrict {
            FunctionCalledOnNull::ReturnsNullOnNullInput
        } else {
            FunctionCalledOnNull::CalledOnNullInput
        }
    }

    fn configuration_parameters(&self) -> &[FunctionDefinitionSetParam] {
        self.configuration()
    }

    fn owner<'db>(&self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError> {
        Ok(metadata(self, database)?.owner())
    }

    fn argument_names<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> impl Iterator<Item = Option<TargetName<'db>>> {
        // `proargnames` holds an empty string for an unnamed argument.
        self.model()
            .proargnames
            .iter()
            .flatten()
            .map(|name| (!name.is_empty()).then(|| TargetName::new(name, true)))
    }

    fn argument_type_names<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = Cow<'db, str>> {
        metadata(self, database)
            .into_iter()
            .flat_map(PgProcMetadata::argument_types)
            .map(|pg_type| Cow::Borrowed(pg_type.typname.as_str()))
    }

    fn return_type_name<'db>(&'db self, database: &'db Self::DB) -> Option<Cow<'db, str>> {
        metadata(self, database)
            .ok()?
            .return_type()
            .map(|pg_type| Cow::Borrowed(pg_type.typname.as_str()))
    }
}
