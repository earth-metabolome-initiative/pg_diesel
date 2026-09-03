//! Implementation of [`SchemaLike`] for [`Schemata`].

use crate::PgDieselDatabase;
use crate::models::Schemata;
use sql_traits::traits::{Metadata, SchemaLike};

impl Metadata for Schemata {
    type Meta = ();
}

impl SchemaLike for Schemata {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.schema_name
    }

    fn authorization(&self) -> Option<&str> {
        Some(&self.schema_owner)
    }
}
