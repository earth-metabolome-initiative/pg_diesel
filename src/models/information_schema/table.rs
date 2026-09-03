//! Table model and related cached queries.

use std::{collections::HashMap, fmt::Display, sync::Arc};

use diesel::{PgConnection, Queryable, QueryableByName, Selectable};

use crate::{
    database::CatalogCache,
    model_metadata::{
        ColumnMetadata, PgForeignKey, PgIndexEntry, PgPolicy, PgTable, TableMetadata,
    },
    models::{CheckConstraint, Column, Triggers},
};

mod cached_queries;
pub(crate) use cached_queries::*;

#[derive(
    Queryable, QueryableByName, PartialEq, Eq, PartialOrd, Ord, Selectable, Debug, Clone, Hash,
)]
#[diesel(table_name = crate::schema::information_schema::tables::tables)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Struct defining the `information_schema.tables` table.
pub struct Table {
    /// The catalog name of the table.
    pub table_catalog: String,
    /// The schema name of the table.
    pub table_schema: String,
    /// The name of the table.
    pub table_name: String,
    /// The type of the table, e.g. `BASE TABLE` or `VIEW`.
    pub table_type: String,
    /// The name of the table that the current table is a temporary table for.
    pub self_referencing_column_name: Option<String>,
    /// The name of the column that is a foreign key to the current table.
    pub reference_generation: Option<String>,
    /// The user-defined type catalog.
    pub user_defined_type_catalog: Option<String>,
    /// The user-defined type schema.
    pub user_defined_type_schema: Option<String>,
    /// The user-defined type name.
    pub user_defined_type_name: Option<String>,
    /// The user-defined type name that the current table is a temporary table
    /// for.
    pub is_insertable_into: String,
    /// Whether the table is typed.
    pub is_typed: String,
    /// Whether the table is updatable.
    pub commit_action: Option<String>,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}.{}`", self.table_schema, self.table_name)
    }
}

impl Table {
    /// Initializes and returns the metadata for the table.
    ///
    /// # Errors
    ///
    /// * If the metadata cannot be loaded from the database.
    #[allow(clippy::too_many_lines)]
    pub fn metadata(
        &self,
        table: &Arc<PgTable>,
        conn: &mut PgConnection,
        denylist_types: &[String],
        cache: &CatalogCache,
    ) -> Result<TableMetadata, diesel::result::Error> {
        let facts = cached_queries::catalog_facts(self, conn)?;
        let attributes = cached_queries::attributes(facts.oid, conn)?;
        let descriptions = cached_queries::descriptions(facts.oid, conn)?;

        let mut sql_metadata = sql_traits::structs::TableMetadata::default();
        let mut column_metadata = HashMap::new();
        for column in cached_queries::columns(self, conn)? {
            let Some(attribute) = attributes
                .iter()
                .find(|attribute| attribute.name == column.column_name)
            else {
                continue;
            };
            let Some(pg_type) = cache.pg_type(attribute.type_oid) else {
                continue;
            };
            if denylist_types.contains(&pg_type.typname) {
                continue;
            }
            let description = descriptions
                .iter()
                .find(|description| description.objsubid == i32::from(attribute.number))
                .cloned();
            let collation = column
                .collation_schema
                .as_deref()
                .zip(column.collation_name.as_deref())
                .and_then(|(schema, name)| cache.collation_is_deterministic(schema, name));
            column_metadata.insert(
                column.column_name.clone(),
                ColumnMetadata::new(Arc::clone(table), description, pg_type.clone(), collation),
            );
            sql_metadata.add_column(Arc::new(column));
        }
        for check_constraint in cached_queries::check_constraints(self, conn)? {
            sql_metadata.add_check_constraint(Arc::new(check_constraint));
        }
        for foreign_key in cached_queries::foreign_keys(self, conn)? {
            let (schema, name) = foreign_key.referenced_relation(conn)?;
            sql_metadata.add_foreign_key(Arc::new(PgForeignKey::new(foreign_key, schema, name)));
        }
        for (index, index_name) in cached_queries::unique_indices(self, conn)? {
            sql_metadata.add_unique_index(Arc::new(PgIndexEntry::new(
                index,
                self.table_schema.clone(),
                index_name,
            )));
        }
        for (index, index_name) in cached_queries::non_unique_indices(self, conn)? {
            sql_metadata.add_index(Arc::new(PgIndexEntry::new(
                index,
                self.table_schema.clone(),
                index_name,
            )));
        }
        let mut primary_key_columns = Vec::new();
        for pk_column in cached_queries::primary_key_columns(self, conn)? {
            primary_key_columns.extend(
                sql_metadata
                    .column_arcs()
                    .filter(|col: &&Arc<Column>| col.as_ref() == &pk_column)
                    .cloned(),
            );
        }
        sql_metadata.set_primary_key(primary_key_columns);

        let triggers = cached_queries::triggers(self, conn)?
            .into_iter()
            .map(|(trigger, oid)| (Arc::new(trigger), oid))
            .collect();

        let policies = cached_queries::policies(facts.oid, conn)?
            .into_iter()
            .map(|policy| Arc::new(PgPolicy::new(policy, Arc::clone(table))))
            .collect();

        sql_metadata.set_rls_enabled(facts.row_security);
        sql_metadata.set_rls_forced(facts.forced_row_security);
        sql_metadata.set_owner(cache.role(facts.owner).map(ToOwned::to_owned));
        sql_metadata.set_inherited_column_names(
            attributes
                .iter()
                .filter(|attribute| attribute.inherited)
                .map(|attribute| attribute.name.clone())
                .collect(),
        );

        let ancestors = cached_queries::parents(facts.oid, conn)?;
        let (parents, partition_root) = if facts.is_partition {
            (Vec::new(), ancestors.into_iter().next())
        } else {
            (ancestors, None)
        };

        Ok(TableMetadata::new(
            sql_metadata,
            descriptions
                .into_iter()
                .find(|description| description.objsubid == 0),
            triggers,
            policies,
            parents,
            partition_root,
            column_metadata,
        ))
    }

    /// Returns the partitioning strategy `pg_partitioned_table` records for
    /// the table, or [`None`] when the table is not partitioned.
    ///
    /// # Errors
    ///
    /// * If the strategy cannot be loaded from the database.
    pub fn partition_strategy(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Option<String>, diesel::result::Error> {
        cached_queries::partition_strategy(cached_queries::catalog_facts(self, conn)?.oid, conn)
    }

    #[must_use]
    /// Returns whether the table is temporary.
    pub fn is_temporary(&self) -> bool {
        self.table_type == "LOCAL TEMPORARY" || self.table_type == "GLOBAL TEMPORARY"
    }

    /// Returns the indices for the table.
    ///
    /// # Errors
    ///
    /// * If the indices cannot be loaded from the database.
    pub fn indices(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<PgIndexEntry>, diesel::result::Error> {
        Ok(indices(self, conn)?
            .into_iter()
            .map(|(index, name)| PgIndexEntry::new(index, self.table_schema.clone(), name))
            .collect())
    }

    /// Returns the primary key columns for the table.
    ///
    /// # Errors
    ///
    /// * If the primary key columns cannot be loaded from the database.
    pub fn primary_key_columns(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<Column>, diesel::result::Error> {
        primary_key_columns(self, conn)
    }

    /// Returns the UNIQUE constraint indices for the table.
    ///
    /// # Errors
    ///
    /// * If the indices cannot be loaded from the database.
    pub fn unique_indices(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<PgIndexEntry>, diesel::result::Error> {
        Ok(unique_indices(self, conn)?
            .into_iter()
            .map(|(index, name)| PgIndexEntry::new(index, self.table_schema.clone(), name))
            .collect())
    }

    /// Returns all tables in the database.
    ///
    /// # Errors
    ///
    /// * If the tables cannot be loaded from the database.
    pub fn load_all(
        conn: &mut PgConnection,
        table_catalog: &str,
        table_schema: &str,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        load_all_tables(table_catalog, table_schema, conn)
    }

    /// Returns the table by name.
    ///
    /// # Errors
    ///
    /// * If the table cannot be loaded from the database.
    pub fn load(
        table_name: &str,
        table_schema: &str,
        table_catalog: &str,
        conn: &mut PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        load_table(conn, table_name, table_schema, table_catalog)
    }

    /// Returns the column by name.
    ///
    /// # Errors
    ///
    /// * If the column cannot be loaded from the database.
    pub fn column_by_name(
        &self,
        column_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Column, diesel::result::Error> {
        column_by_name(self, column_name, conn)
    }

    /// Returns the check constraints for the table.
    ///
    /// # Errors
    ///
    /// * If the check constraints cannot be loaded from the database.
    pub fn check_constraints(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<CheckConstraint>, diesel::result::Error> {
        check_constraints(self, conn)
    }

    /// Returns the list of Triggers associates to the current table.
    ///
    /// # Errors
    ///
    /// * If the triggers cannot be loaded from the database.
    pub fn triggers(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<(Triggers, Option<u32>)>, diesel::result::Error> {
        triggers(self, conn)
    }
}

impl AsRef<Table> for Table {
    fn as_ref(&self) -> &Table {
        self
    }
}
