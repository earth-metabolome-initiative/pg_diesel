//! Submodule providing the `ColumnMetadata` struct for a [`Column`](crate::models::Column) model.

use std::rc::Rc;

use crate::models::{PgDescription, PgType, Table};

#[derive(Clone, Debug)]
/// Rich metadata about a `PostgreSQL` table column.
///
/// This struct wraps a column model with additional metadata loaded from
/// related system catalog tables, including:
/// - The table that owns the column
/// - The resolved `PostgreSQL` type ([`PgType`]) for the column
/// - Column description from `pg_catalog.pg_description`
///
/// This metadata is constructed during
/// [`PgDieselDatabase`](crate::database::PgDieselDatabase) building and cached for
/// efficient access via the `ColumnLike`
/// trait.
///
/// ## Type Resolution
///
/// The `pg_type` field contains the fully resolved type information from
/// `pg_catalog.pg_type`, not just the type name from
/// `information_schema.columns`. This provides access to:
/// - Internal type name (`typname`)
/// - Type category and properties
/// - Array element types for array types
pub struct ColumnMetadata {
    /// The table the column belongs to.
    table: Rc<Table>,
    /// The description of the column, if any.
    description: Option<PgDescription>,
    /// The associated `PgType`.
    pg_type: PgType,
}

impl ColumnMetadata {
    /// Creates a new `ColumnMetadata` instance.
    #[must_use]
    pub fn new(table: Rc<Table>, description: Option<PgDescription>, pg_type: PgType) -> Self {
        Self {
            table,
            description,
            pg_type,
        }
    }

    /// Returns the table the column belongs to.
    #[must_use]
    pub fn table(&self) -> &Table {
        self.table.as_ref()
    }

    /// Returns the description of the column, if any.
    #[must_use]
    pub fn description(&self) -> Option<&PgDescription> {
        self.description.as_ref()
    }

    /// Returns the associated [`PgType`].
    #[must_use]
    pub fn pg_type(&self) -> &PgType {
        &self.pg_type
    }

    /// Returns the normalized data type of the column.
    #[must_use]
    pub fn normalized_data_type(&self) -> String {
        self.pg_type.typname.clone()
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

    fn dummy_pg_type() -> PgType {
        PgType {
            oid: 1,
            typname: "int4".to_string(),
            typnamespace: 1,
            typowner: 1,
            typlen: 4,
            typbyval: true,
            typtype: "b".to_string(),
            typcategory: "N".to_string(),
            typispreferred: false,
            typisdefined: true,
            typdelim: ",".to_string(),
            typrelid: 0,
            typelem: 0,
            typarray: 0,
            typinput: 0,
            typoutput: 0,
            typreceive: 0,
            typsend: 0,
            typmodin: 0,
            typmodout: 0,
            typanalyze: 0,
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            typsubscript: 0,
            typalign: "i".to_string(),
            typstorage: "p".to_string(),
            typnotnull: false,
            typbasetype: 0,
            typtypmod: -1,
            typndims: 0,
            typcollation: 0,
            typdefaultbin: None,
            typdefault: None,
            typacl: None,
        }
    }

    #[test]
    fn test_column_metadata() {
        let table = Rc::new(dummy_table());
        let pg_type = dummy_pg_type();
        let description = Some(PgDescription {
            objoid: 1,
            classoid: 1,
            objsubid: 1,
            description: "desc".to_string(),
        });

        let metadata = ColumnMetadata::new(Rc::clone(&table), description, pg_type);

        assert_eq!(metadata.table().table_name, "table");
        assert_eq!(metadata.description().unwrap().description, "desc");
        assert_eq!(metadata.pg_type().typname, "int4");
        assert_eq!(metadata.normalized_data_type(), "int4");
    }
}
