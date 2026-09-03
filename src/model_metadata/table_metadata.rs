//! Submodule providing the `TableMetadata` struct for a [`PgTable`] model.

use std::{collections::HashMap, sync::Arc};

use crate::{
    model_metadata::{ColumnMetadata, PgForeignKey, PgIndexEntry, PgPolicy, PgTable},
    models::{CheckConstraint, Column, PgDescription, Triggers},
};

/// The schema and name a table is recorded under, as the catalogs spell them.
pub type TableIdentity = (String, String);

#[derive(Clone, Debug)]
/// Rich metadata about a `PostgreSQL` table.
pub struct TableMetadata {
    /// The underlying table metadata.
    metadata: sql_traits::structs::TableMetadata<PgTable>,
    /// The description of the table, if any.
    description: Option<PgDescription>,
    /// The triggers defined on the table, along with the OID of the function they call.
    triggers: Vec<(Arc<Triggers>, Option<u32>)>,
    /// The policies defined on the table.
    policies: Vec<Arc<PgPolicy>>,
    /// The tables this one inherits from through `INHERITS`.
    parents: Vec<TableIdentity>,
    /// The partitioned table this one is a partition of.
    partition_root: Option<TableIdentity>,
    /// The metadata of each column, by the name the column is recorded under.
    column_metadata: HashMap<String, ColumnMetadata>,
}

impl TableMetadata {
    /// Creates a new `TableMetadata` instance.
    #[must_use]
    pub fn new(
        metadata: sql_traits::structs::TableMetadata<PgTable>,
        description: Option<PgDescription>,
        triggers: Vec<(Arc<Triggers>, Option<u32>)>,
        policies: Vec<Arc<PgPolicy>>,
        parents: Vec<TableIdentity>,
        partition_root: Option<TableIdentity>,
        column_metadata: HashMap<String, ColumnMetadata>,
    ) -> Self {
        Self {
            metadata,
            description,
            triggers,
            policies,
            parents,
            partition_root,
            column_metadata,
        }
    }

    /// Returns the metadata recorded for the column of the given name.
    #[must_use]
    pub fn column_metadata(&self, column_name: &str) -> Option<&ColumnMetadata> {
        self.column_metadata.get(column_name)
    }

    /// Returns an iterator over the references of columns of the table.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.metadata.columns()
    }

    /// Returns an iterator over the columns the table declares itself.
    pub fn local_columns(&self) -> impl Iterator<Item = &Column> {
        self.metadata.local_columns()
    }

    /// Returns an iterator over the Arc of columns of the table.
    pub fn column_arcs(&self) -> impl Iterator<Item = &Arc<Column>> {
        self.metadata.column_arcs()
    }

    /// Returns a slice of Arc of columns of the table.
    #[must_use]
    pub fn column_arc_slice(&self) -> &[Arc<Column>] {
        self.metadata.column_arc_slice()
    }

    /// Returns an iterator over the check constraints of the table.
    pub fn check_constraints(&self) -> impl Iterator<Item = &CheckConstraint> {
        self.metadata.check_constraints()
    }

    /// Returns an iterator over the Arc of check constraints of the table.
    pub fn check_constraint_arcs(&self) -> impl Iterator<Item = &Arc<CheckConstraint>> {
        self.metadata.check_constraint_arcs()
    }

    /// Returns an iterator over the indices of the table.
    pub fn indices(&self) -> impl Iterator<Item = &PgIndexEntry> {
        self.metadata.indices()
    }

    /// Returns an iterator over the Arc of indices of the table.
    pub fn index_arcs(&self) -> impl Iterator<Item = &Arc<PgIndexEntry>> {
        self.metadata.index_arcs()
    }

    /// Returns an iterator over the unique indices of the table.
    pub fn unique_indices(&self) -> impl Iterator<Item = &PgIndexEntry> {
        self.metadata.unique_indices()
    }

    /// Returns an iterator over the Arc of unique indices of the table.
    pub fn unique_index_arcs(&self) -> impl Iterator<Item = &Arc<PgIndexEntry>> {
        self.metadata.unique_index_arcs()
    }

    /// Returns an iterator over the foreign keys of the table.
    pub fn foreign_keys(&self) -> impl Iterator<Item = &PgForeignKey> {
        self.metadata.foreign_keys()
    }

    /// Returns an iterator over the Arc of foreign keys of the table.
    pub fn foreign_key_arcs(&self) -> impl Iterator<Item = &Arc<PgForeignKey>> {
        self.metadata.foreign_key_arcs()
    }

    /// Returns an iterator over the columns composing the primary key of the
    /// table.
    pub fn primary_key_columns(&self) -> impl Iterator<Item = &Column> {
        self.metadata.primary_key_columns()
    }

    /// Returns the description of the table, if any.
    #[must_use]
    pub fn description(&self) -> Option<&PgDescription> {
        self.description.as_ref()
    }

    /// Returns an iterator over the triggers of the table.
    pub fn triggers(&self) -> impl Iterator<Item = &(Arc<Triggers>, Option<u32>)> {
        self.triggers.iter()
    }

    /// Returns an iterator over the policies of the table.
    pub fn policies(&self) -> impl Iterator<Item = &Arc<PgPolicy>> {
        self.policies.iter()
    }

    /// Returns whether the table has row-level security enabled.
    #[must_use]
    pub fn row_security(&self) -> bool {
        self.metadata.rls_enabled()
    }

    /// Returns whether the table has row-level security forced.
    #[must_use]
    pub fn forced_row_security(&self) -> bool {
        self.metadata.rls_forced()
    }

    /// Returns the role owning the table.
    #[must_use]
    pub fn owner(&self) -> Option<&str> {
        self.metadata.owner()
    }

    /// Returns the tables this one inherits from through `INHERITS`.
    pub fn parents(&self) -> impl Iterator<Item = &TableIdentity> {
        self.parents.iter()
    }

    /// Returns the partitioned table this one is a partition of.
    #[must_use]
    pub fn partition_root(&self) -> Option<&TableIdentity> {
        self.partition_root.as_ref()
    }
}
