//! Submodule providing [`PgIndexEntry`], the index a `PgDieselDatabase` holds.

use crate::models::PgIndex;

/// A `pg_index` row paired with the name and schema its `indexrelid` names.
///
/// `IndexLike` asks for both without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgIndexEntry {
    /// The `pg_catalog.pg_index` row describing the index.
    model: PgIndex,
    /// The schema the index lives in.
    schema: String,
    /// The name of the index.
    name: String,
}

impl PgIndexEntry {
    /// Pairs a `pg_index` row with the relation name its `indexrelid` names.
    #[must_use]
    pub fn new(model: PgIndex, schema: String, name: String) -> Self {
        Self {
            model,
            schema,
            name,
        }
    }

    /// Returns the `pg_catalog.pg_index` row describing the index.
    #[must_use]
    pub fn model(&self) -> &PgIndex {
        &self.model
    }

    /// Returns the schema the index lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of the index.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the metadata for the index.
    ///
    /// # Arguments
    ///
    /// * `table` - A reference-counted pointer to the table this index belongs
    ///   to.
    /// * `conn` - A mutable reference to a `PgConnection`
    ///
    /// # Errors
    ///
    /// * If an error occurs while loading the metadata from the database
    pub fn metadata(
        &self,
        table: std::sync::Arc<crate::model_metadata::PgTable>,
        conn: &mut diesel::PgConnection,
    ) -> Result<sql_traits::structs::metadata::UniqueIndexMetadata<Self>, diesel::result::Error>
    {
        Ok(sql_traits::structs::metadata::UniqueIndexMetadata::new(
            self.model.index_expression(conn)?,
            table,
        ))
    }
}
