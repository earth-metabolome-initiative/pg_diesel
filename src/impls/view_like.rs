//! Implementation of [`ViewLike`] for [`PgViewDefinition`].

use sql_traits::traits::{Metadata, ViewLike};
use sqlparser::ast::Query;

use crate::{PgDieselDatabase, model_metadata::PgViewDefinition};

impl Metadata for PgViewDefinition {
    type Meta = ();
}

impl ViewLike for PgViewDefinition {
    type DB = PgDieselDatabase;

    fn view_name(&self) -> &str {
        self.name()
    }

    fn view_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn view_schema(&self) -> Option<&str> {
        Some(self.schema())
    }

    fn view_schema_is_quoted(&self) -> bool {
        true
    }

    fn is_materialized(&self) -> bool {
        PgViewDefinition::is_materialized(self)
    }

    fn definition(&self) -> &Query {
        PgViewDefinition::definition(self)
    }

    fn declared_column_names(&self) -> &[(String, bool)] {
        // `pg_get_viewdef` does not reconstruct the declared column list.
        &[]
    }
}
