//! Submodule providing the `TableMetadata` struct for a [`Table`](crate::models::Table) model.

use std::sync::Arc;

use crate::models::{
    CheckConstraint, Column, KeyColumnUsage, PgDescription, PgIndex, PgPolicyTable, Triggers,
};

#[derive(Clone, Debug)]
/// Rich metadata about a `PostgreSQL` table.
///
/// This struct wraps a table model with additional metadata loaded from related
/// system catalog tables, including:
/// - Columns belonging to the table
/// - Check constraints defined on the table
/// - Unique indexes (including primary key)
/// - Foreign keys referencing other tables
/// - Table description from `pg_catalog.pg_description`
/// - Row Security Policies
///
/// This metadata is constructed during
/// [`PgDieselDatabase`](crate::database::PgDieselDatabase) building and cached for
/// efficient access via the `TableLike` trait.
pub struct TableMetadata {
    /// The underlying table metadata.
    metadata: sql_traits::structs::TableMetadata<crate::models::Table>,
    /// The description of the table, if any.
    description: Option<PgDescription>,
    /// The triggers defined on the table, along with the OID of the function they call.
    triggers: Vec<(Arc<Triggers>, Option<u32>)>,
    /// The policies defined on the table.
    policies: Vec<Arc<PgPolicyTable>>,
    /// Whether the table has row-level security enabled.
    row_security: bool,
    /// Whether the table has row-level security forced.
    forced_row_security: bool,
}

impl TableMetadata {
    /// Creates a new `TableMetadata` instance.
    #[must_use]
    pub fn new(
        metadata: sql_traits::structs::TableMetadata<crate::models::Table>,
        description: Option<PgDescription>,
        triggers: Vec<(Arc<Triggers>, Option<u32>)>,
        policies: Vec<Arc<PgPolicyTable>>,
        row_security: bool,
        forced_row_security: bool,
    ) -> Self {
        Self {
            metadata,
            description,
            triggers,
            policies,
            row_security,
            forced_row_security,
        }
    }

    /// Returns an iterator over the references of columns of the table.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.metadata.columns()
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
    pub fn indices(&self) -> impl Iterator<Item = &PgIndex> {
        self.metadata.indices()
    }

    /// Returns an iterator over the Arc of indices of the table.
    pub fn index_arcs(&self) -> impl Iterator<Item = &Arc<PgIndex>> {
        self.metadata.index_arcs()
    }

    /// Returns an iterator over the unique indices of the table.
    pub fn unique_indices(&self) -> impl Iterator<Item = &PgIndex> {
        self.metadata.unique_indices()
    }

    /// Returns an iterator over the Arc of unique indices of the table.
    pub fn unique_index_arcs(&self) -> impl Iterator<Item = &Arc<PgIndex>> {
        self.metadata.unique_index_arcs()
    }

    /// Returns an iterator over the foreign keys of the table.
    pub fn foreign_keys(&self) -> impl Iterator<Item = &KeyColumnUsage> {
        self.metadata.foreign_keys()
    }

    /// Returns an iterator over the Arc of foreign keys of the table.
    pub fn foreign_key_arcs(&self) -> impl Iterator<Item = &Arc<KeyColumnUsage>> {
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
    pub fn policies(&self) -> impl Iterator<Item = &Arc<PgPolicyTable>> {
        self.policies.iter()
    }

    /// Returns whether the table has row-level security enabled.
    #[must_use]
    pub fn row_security(&self) -> bool {
        self.row_security
    }

    /// Returns whether the table has row-level security forced.
    #[must_use]
    pub fn forced_row_security(&self) -> bool {
        self.forced_row_security
    }
}
