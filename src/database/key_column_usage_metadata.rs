//! Metadata for foreign key relationships.

use std::sync::Arc;

use crate::{
    model_metadata::PgTable,
    models::{Column, ReferentialConstraint},
};

#[derive(Debug, Clone)]
/// Struct collecting metadata about a foreign key represented by a
/// [`PgForeignKey`](crate::model_metadata::PgForeignKey) entry.
pub struct KeyColumnUsageMetadata {
    /// The columns in the referenced table that the foreign key points to.
    referenced_columns: Vec<Column>,
    /// The table that contains the foreign key.
    host_table: Arc<PgTable>,
    /// The columns in the host table that are part of the foreign key.
    host_columns: Vec<Column>,
    /// The referential constraint associated with the foreign key.
    referential_constraint: ReferentialConstraint,
}

impl KeyColumnUsageMetadata {
    /// Creates a new `KeyColumnUsageMetadata` instance.
    pub(crate) fn new(
        referenced_columns: Vec<Column>,
        host_table: Arc<PgTable>,
        host_columns: Vec<Column>,
        referential_constraint: ReferentialConstraint,
    ) -> Self {
        Self {
            referenced_columns,
            host_table,
            host_columns,
            referential_constraint,
        }
    }

    /// Returns a reference to the table that contains the foreign key.
    #[must_use]
    pub fn host_table(&self) -> &PgTable {
        &self.host_table
    }

    /// Returns a reference to the columns in the referenced table that the
    /// foreign key points to.
    #[must_use]
    pub fn referenced_columns(&self) -> &[Column] {
        &self.referenced_columns
    }

    /// Returns a reference to the columns in the host table that are part of
    /// the foreign key.
    #[must_use]
    pub fn host_columns(&self) -> &[Column] {
        &self.host_columns
    }

    /// Returns a reference to the referential constraint associated with the
    /// foreign key.
    #[must_use]
    pub fn referential_constraint(&self) -> &ReferentialConstraint {
        &self.referential_constraint
    }

    /// Returns whether the foreign key has an `ON DELETE CASCADE` rule.
    #[must_use]
    pub fn on_delete_cascade(&self) -> bool {
        self.referential_constraint.on_delete_cascade()
    }

    /// Returns the match kind of the foreign key.
    #[must_use]
    pub fn match_kind(&self) -> sqlparser::ast::ConstraintReferenceMatchKind {
        self.referential_constraint.match_kind()
    }
}
