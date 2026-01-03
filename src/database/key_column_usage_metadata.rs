//! Metadata for foreign key relationships.
//!
//! This module provides [`KeyColumnUsageMetadata`], which encapsulates complete
//! information about a foreign key relationship, including:
//! - The referencing (host) table and columns
//! - The referenced (target) table and columns
//! - The referential constraint rules (ON DELETE, ON UPDATE, MATCH)
//!
//! This metadata is used by the [`PgDatabase`](crate::database::PgDatabase) to
//! provide rich foreign key introspection through the `sql_traits` trait
//! system.

use std::rc::Rc;

use crate::models::{Column, ReferentialConstraint, Table};

#[derive(Debug, Clone)]
/// Struct collecting metadata about a foreign key represented by a
/// [`KeyColumnUsage`](crate::models::KeyColumnUsage) entry.
pub struct KeyColumnUsageMetadata {
    /// The table that the foreign key references.
    referenced_table: Table,
    /// The columns in the referenced table that the foreign key points to.
    referenced_columns: Vec<Column>,
    /// The table that contains the foreign key.
    host_table: Rc<Table>,
    /// The columns in the host table that are part of the foreign key.
    host_columns: Vec<Column>,
    /// The referential constraint associated with the foreign key.
    referential_constraint: ReferentialConstraint,
}

impl KeyColumnUsageMetadata {
    /// Creates a new `KeyColumnUsageMetadata` instance from the given
    /// [`KeyColumnUsage`](crate::models::KeyColumnUsage), and connection to the
    /// database.
    ///
    /// # Arguments
    ///
    /// * `referenced_table` - The table that the foreign key references.
    /// * `referenced_columns` - The columns in the referenced table that the
    ///   foreign key points to.
    /// * `host_table` - The table that contains the foreign key.
    /// * `host_columns` - The columns in the host table that are part of the
    ///   foreign key.
    /// * `referential_constraint` - The referential constraint associated with
    ///   the foreign key.
    pub(crate) fn new(
        referenced_table: Table,
        referenced_columns: Vec<Column>,
        host_table: Rc<Table>,
        host_columns: Vec<Column>,
        referential_constraint: ReferentialConstraint,
    ) -> Self {
        Self {
            referenced_table,
            referenced_columns,
            host_table,
            host_columns,
            referential_constraint,
        }
    }

    /// Returns a reference to the table that the foreign key references.
    #[must_use]
    pub fn referenced_table(&self) -> &Table {
        &self.referenced_table
    }

    /// Returns a reference to the table that contains the foreign key.
    #[must_use]
    pub fn host_table(&self) -> &Table {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_table() -> Table {
        Table {
            table_catalog: "db".to_string(),
            table_schema: "schema".to_string(),
            table_name: "table".to_string(),
            table_type: "BASE TABLE".to_string(),
            self_referencing_column_name: None,
            reference_generation: None,
            user_defined_type_catalog: None,
            user_defined_type_schema: None,
            user_defined_type_name: None,
            is_insertable_into: "YES".to_string(),
            is_typed: "NO".to_string(),
            commit_action: None,
        }
    }

    fn dummy_referential_constraint() -> ReferentialConstraint {
        ReferentialConstraint {
            constraint_catalog: "db".to_string(),
            constraint_schema: "schema".to_string(),
            constraint_name: "constraint".to_string(),
            unique_constraint_catalog: None,
            unique_constraint_schema: None,
            unique_constraint_name: None,
            match_option: "SIMPLE".to_string(),
            update_rule: "NO ACTION".to_string(),
            delete_rule: "CASCADE".to_string(),
        }
    }

    #[test]
    fn test_key_column_usage_metadata() {
        let referenced_table = dummy_table();
        let host_table = Rc::new(dummy_table());
        let referential_constraint = dummy_referential_constraint();

        let metadata = KeyColumnUsageMetadata::new(
            referenced_table,
            vec![],
            host_table,
            vec![],
            referential_constraint,
        );

        assert_eq!(metadata.referenced_table().table_name, "table");
        assert_eq!(metadata.host_table().table_name, "table");
        assert!(metadata.referenced_columns().is_empty());
        assert!(metadata.host_columns().is_empty());
        assert_eq!(
            metadata.referential_constraint().constraint_name,
            "constraint"
        );
        assert!(metadata.on_delete_cascade());
        assert!(matches!(
            metadata.match_kind(),
            sqlparser::ast::ConstraintReferenceMatchKind::Simple
        ));
    }
}
