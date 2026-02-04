//! Submodule defining the cached queries methods used in the [`Table`] struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgConnection,
    QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::models::{
    CheckConstraint, Column, KeyColumnUsage, PgDescription, PgIndex, PgTrigger, Table, Triggers,
};

/// Loads all tables from the information schema for the given catalog and schema.
pub(crate) fn load_all_tables(
    table_catalog: &str,
    table_schema: &str,
    conn: &mut PgConnection,
) -> Result<Vec<Table>, diesel::result::Error> {
    use crate::schema::information_schema::tables::tables;
    tables::table
        .filter(tables::table_catalog.eq(table_catalog))
        .filter(tables::table_schema.eq(table_schema))
        .filter(tables::table_name.ne("__diesel_schema_migrations"))
        .order_by(tables::table_name)
        .select(Table::as_select())
        .load::<Table>(conn)
}

/// Loads a specific table by name, schema, and catalog.
pub(crate) fn load_table(
    conn: &mut PgConnection,
    table_name: &str,
    table_schema: &str,
    table_catalog: &str,
) -> Result<Table, diesel::result::Error> {
    use crate::schema::information_schema::tables::tables;
    tables::table
        .filter(tables::table_name.eq(table_name))
        .filter(tables::table_schema.eq(table_schema))
        .filter(tables::table_catalog.eq(table_catalog))
        .first::<Table>(conn)
}

/// Returns the columns of the table.
pub(crate) fn columns(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<Column>, diesel::result::Error> {
    use crate::schema::information_schema::columns::columns;
    columns::table
        .filter(columns::table_name.eq(&table.table_name))
        .filter(columns::table_schema.eq(&table.table_schema))
        .filter(columns::table_catalog.eq(&table.table_catalog))
        .order_by(columns::ordinal_position)
        .load::<Column>(conn)
}

/// Returns the columns that are part of the primary key for the given table.
///
/// # Arguments
///
/// * `table` - The table for which to retrieve the primary key columns.
/// * `conn` - The database connection.
///
/// # Errors
///
/// * If the columns cannot be loaded from the database.
pub fn primary_key_columns(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<Column>, diesel::result::Error> {
    use crate::schema::information_schema::{
        columns::columns, key_column_usage::key_column_usage, table_constraints::table_constraints,
    };
    key_column_usage::table
        .inner_join(
            columns::table.on(key_column_usage::table_name
                .nullable()
                .eq(columns::table_name.nullable())
                .and(
                    key_column_usage::table_schema
                        .nullable()
                        .eq(columns::table_schema.nullable()),
                )
                .and(
                    key_column_usage::table_catalog
                        .nullable()
                        .eq(columns::table_catalog.nullable()),
                )
                .and(
                    key_column_usage::column_name
                        .nullable()
                        .eq(columns::column_name.nullable()),
                )),
        )
        .inner_join(
            table_constraints::table.on(key_column_usage::constraint_name
                .nullable()
                .eq(table_constraints::constraint_name.nullable())
                .and(
                    key_column_usage::constraint_schema
                        .nullable()
                        .eq(table_constraints::constraint_schema.nullable()),
                )
                .and(
                    key_column_usage::constraint_catalog
                        .nullable()
                        .eq(table_constraints::constraint_catalog.nullable()),
                )
                .and(
                    key_column_usage::table_name
                        .nullable()
                        .eq(table_constraints::table_name.nullable()),
                )
                .and(
                    key_column_usage::table_schema
                        .nullable()
                        .eq(table_constraints::table_schema.nullable()),
                )
                .and(
                    key_column_usage::table_catalog
                        .nullable()
                        .eq(table_constraints::table_catalog.nullable()),
                )),
        )
        .filter(key_column_usage::table_name.eq(&table.table_name))
        .filter(key_column_usage::table_schema.eq(&table.table_schema))
        .filter(key_column_usage::table_catalog.eq(&table.table_catalog))
        .filter(table_constraints::constraint_type.eq("PRIMARY KEY"))
        .select(Column::as_select())
        .load::<Column>(conn)
}

/// Returns the foreign keys of the table.
pub(crate) fn foreign_keys(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<KeyColumnUsage>, diesel::result::Error> {
    use crate::schema::information_schema::{
        key_column_usage::key_column_usage, referential_constraints::referential_constraints,
    };
    key_column_usage::table
        .inner_join(
            referential_constraints::table.on(key_column_usage::constraint_name
                .eq(referential_constraints::constraint_name)
                .and(
                    key_column_usage::constraint_schema
                        .eq(referential_constraints::constraint_schema),
                )
                .and(
                    key_column_usage::constraint_catalog
                        .eq(referential_constraints::constraint_catalog),
                )),
        )
        .filter(key_column_usage::table_name.eq(&table.table_name))
        .filter(key_column_usage::table_schema.eq(&table.table_schema))
        .filter(key_column_usage::table_catalog.eq(&table.table_catalog))
        .filter(key_column_usage::ordinal_position.eq(1))
        .order_by((
            key_column_usage::constraint_catalog,
            key_column_usage::constraint_schema,
            key_column_usage::constraint_name,
            key_column_usage::ordinal_position,
        ))
        .select(KeyColumnUsage::as_select())
        .load::<KeyColumnUsage>(conn)
}

/// Returns the unique indices of the table.
pub(crate) fn unique_indices(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<PgIndex>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_index::pg_index};

    let (pg_class1, pg_class2) = diesel::alias!(pg_class as pg_class1, pg_class as pg_class2);

    pg_index::table
        .inner_join(pg_class1.on(pg_class1.field(pg_class::oid).eq(pg_index::indexrelid)))
        .inner_join(pg_class2.on(pg_class2.field(pg_class::oid).eq(pg_index::indrelid)))
        .filter(
            pg_class2
                .field(pg_class::relname)
                .eq(&table.table_name)
                .and(
                    pg_class2
                        .field(pg_class::relnamespace)
                        .eq(pg_class1.field(pg_class::relnamespace)),
                ),
        )
        .filter(pg_index::indisunique.eq(true))
        .select(PgIndex::as_select())
        .load::<PgIndex>(conn)
}

/// Returns all indices of the table.
pub(crate) fn indices(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<PgIndex>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_index::pg_index};

    let (pg_class1, pg_class2) = diesel::alias!(pg_class as pg_class1, pg_class as pg_class2);

    pg_index::table
        .inner_join(pg_class1.on(pg_class1.field(pg_class::oid).eq(pg_index::indexrelid)))
        .inner_join(pg_class2.on(pg_class2.field(pg_class::oid).eq(pg_index::indrelid)))
        .filter(
            pg_class2
                .field(pg_class::relname)
                .eq(&table.table_name)
                .and(
                    pg_class2
                        .field(pg_class::relnamespace)
                        .eq(pg_class1.field(pg_class::relnamespace)),
                ),
        )
        .select(PgIndex::as_select())
        .load::<PgIndex>(conn)
}

/// Returns the check constraints of the table.
pub(crate) fn check_constraints(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<CheckConstraint>, diesel::result::Error> {
    use crate::schema::information_schema::{
        check_constraints::check_constraints, table_constraints::table_constraints,
    };

    check_constraints::table
        .inner_join(
            table_constraints::table.on(check_constraints::constraint_name
                .eq(table_constraints::constraint_name)
                .and(check_constraints::constraint_schema.eq(table_constraints::constraint_schema))
                .and(
                    check_constraints::constraint_catalog.eq(table_constraints::constraint_catalog),
                )),
        )
        .filter(table_constraints::table_name.eq(&table.table_name))
        .filter(table_constraints::table_schema.eq(&table.table_schema))
        .filter(table_constraints::table_catalog.eq(&table.table_catalog))
        .select(CheckConstraint::as_select())
        .load::<CheckConstraint>(conn)
}

/// Returns the column with the given name from the table.
pub(crate) fn column_by_name(
    table: &Table,
    column_name: &str,
    conn: &mut PgConnection,
) -> Result<Column, diesel::result::Error> {
    use crate::schema::information_schema::columns::columns;
    columns::table
        .filter(columns::table_name.eq(&table.table_name))
        .filter(columns::table_schema.eq(&table.table_schema))
        .filter(columns::table_catalog.eq(&table.table_catalog))
        .filter(columns::column_name.eq(column_name))
        .first::<Column>(conn)
}

/// Returns the description of the table from `pg_description`.
pub(super) fn pg_description(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<PgDescription, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_attribute::pg_attribute, pg_class::pg_class, pg_description::pg_description,
        pg_namespace::pg_namespace,
    };

    pg_description::table
        .inner_join(pg_attribute::table.on(pg_description::objoid.eq(pg_attribute::attrelid)))
        .inner_join(pg_class::table.on(pg_attribute::attrelid.eq(pg_class::oid)))
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::relname.eq(&table.table_name))
        .filter(pg_namespace::nspname.eq(&table.table_schema))
        .filter(pg_attribute::attname.eq(&table.table_name))
        .select(PgDescription::as_select())
        .first::<PgDescription>(conn)
}

/// Returns the triggers of the table, along with the OID of the function they call.
pub(crate) fn triggers(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<(Triggers, Option<u32>)>, diesel::result::Error> {
    use crate::schema::information_schema::triggers::triggers;
    use crate::schema::pg_catalog::{
        pg_class::pg_class, pg_namespace::pg_namespace, pg_trigger::pg_trigger,
    };

    let is_triggers = triggers::table
        .filter(triggers::event_object_catalog.eq(&table.table_catalog))
        .filter(triggers::event_object_schema.eq(&table.table_schema))
        .filter(triggers::event_object_table.eq(&table.table_name))
        .select(Triggers::as_select())
        .load::<Triggers>(conn)?;

    let pg_triggers = pg_trigger::table
        .inner_join(pg_class::table.on(pg_trigger::tgrelid.eq(pg_class::oid)))
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::relname.eq(&table.table_name))
        .filter(pg_namespace::nspname.eq(&table.table_schema))
        .select(PgTrigger::as_select())
        .load::<PgTrigger>(conn)?;

    let mut result = Vec::new();
    for trig in is_triggers {
        let name = trig.trigger_name.as_deref().unwrap_or("");
        let oid = pg_triggers
            .iter()
            .find(|t| t.tgname == name)
            .map(|t| t.tgfoid);
        result.push((trig, oid));
    }
    Ok(result)
}

/// Returns the policies associated with the table.
pub(crate) fn policies(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<crate::models::PgPolicyTable>, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_class::pg_class, pg_namespace::pg_namespace, pg_policy::pg_policy,
    };

    // First find Table OID reliably
    let table_oid: u32 = pg_class::table
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::relname.eq(&table.table_name))
        .filter(pg_namespace::nspname.eq(&table.table_schema))
        .select(pg_class::oid)
        .first(conn)?;

    pg_policy::table
        .filter(pg_policy::polrelid.eq(table_oid))
        .select(crate::models::PgPolicyTable::as_select())
        .load(conn)
}

/// Returns the Row Level Security settings for the table.
pub(crate) fn pg_class(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<(bool, bool), diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_namespace::pg_namespace};

    pg_class::table
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::relname.eq(&table.table_name))
        .filter(pg_namespace::nspname.eq(&table.table_schema))
        .select((pg_class::relrowsecurity, pg_class::relforcerowsecurity))
        .first(conn)
}
