//! Builder pattern for constructing a [`PgDieselDatabase`] instance.

use std::{collections::HashMap, ops::ControlFlow, sync::Arc};

use diesel::PgConnection;
use sql_traits::{
    errors::LookupError, structs::generic_db::GenericDBBuilder, traits::FunctionLike,
    utils::identifier_resolution::identifiers_match,
};
use sqlparser::{
    ast::{
        Expr, Grantee, GranteeName, GranteesType, Ident, ObjectName, ObjectNamePart, Owner,
        visit_expressions,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::{
    PgDieselDatabase,
    database::CatalogCache,
    dialect::PostgresDialect,
    impls::{RoleColumnGrantsMetadata, RoleTableGrantsMetadata, string_to_action},
    model_metadata::{
        PgFunction, PgPolicy, PgTable, PgViewDefinition, TriggerMetadata, UnknownPartitionStrategy,
        ViewDefinitionError,
    },
    models::{
        Column, PgMatview, PgProc, PgRole, PgView, RoleColumnGrants, RoleTableGrants, Schemata,
        Table,
    },
};

/// Returns the loaded functions `expression` calls, each once.
///
/// A call is matched on the last part of its name, so a call written with a
/// schema qualifier and one written without resolve to the same function.
fn called_functions(
    expression: Option<&Expr>,
    functions: &[Arc<PgFunction>],
) -> Vec<Arc<PgFunction>> {
    let Some(expression) = expression else {
        return Vec::new();
    };

    let mut called: Vec<Arc<PgFunction>> = Vec::new();
    let _: ControlFlow<()> = visit_expressions(expression, |expr| {
        if let Expr::Function(function) = expr
            && let Some(ObjectNamePart::Identifier(ident)) = function.name.0.last()
        {
            called.extend(
                functions
                    .iter()
                    .filter(|candidate| {
                        identifiers_match(
                            candidate.name(),
                            candidate.name_is_quoted(),
                            &ident.value,
                            ident.quote_style.is_some(),
                        )
                    })
                    .cloned(),
            );
        }
        ControlFlow::Continue(())
    });

    called.sort_unstable_by_key(|function| function.oid());
    called.dedup_by_key(|function| function.oid());
    called
}

#[derive(Default)]
/// Builder for constructing a [`PgDieselDatabase`] instance from `PostgreSQL` metadata.
pub struct PgDieselDatabaseBuilder<'conn> {
    /// Connection to the `PostgreSQL` database.
    connection: Option<&'conn mut PgConnection>,
    /// The catalog (database) name to filter by.
    catalog: Option<String>,
    /// The schema names to include.
    schemas: Vec<String>,
    /// Types denylist.
    denylist_types: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
/// Errors that can occur when building a [`PgDieselDatabase`] instance.
pub enum PgDatabaseBuildError {
    #[error("Missing required builder attribute: {0}")]
    /// An attribute was missing.
    MissingAttribute(&'static str),
    #[error("Diesel error: {0}")]
    /// An error occurred while querying the database schema.
    Diesel(#[from] diesel::result::Error),
    #[error("Duplicate denylisted type: {0}")]
    /// A deny-listed type was inserted multiple times.
    DuplicateDenylistedType(String),
    #[error("Unknown partitioning strategy: {0}")]
    /// A table reported a partitioning strategy this crate does not know.
    UnknownPartitionStrategy(#[from] UnknownPartitionStrategy),
    #[error("View definition: {0}")]
    /// A view could not be read from the catalogs.
    ViewDefinition(#[from] ViewDefinitionError),
    #[error("Lookup error: {0}")]
    /// Adding an object left the database unable to resolve a name.
    Lookup(#[from] LookupError),
}

impl<'conn> PgDieselDatabaseBuilder<'conn> {
    /// Sets the `PostgreSQL` connection to use for building the `PgDieselDatabase`.
    #[must_use]
    pub fn connection(mut self, connection: &'conn mut PgConnection) -> Self {
        self.connection = Some(connection);
        self
    }

    /// Sets the catalog (database) name to filter by.
    #[must_use]
    pub fn catalog<S: AsRef<str>>(mut self, catalog: S) -> Self {
        self.catalog = Some(catalog.as_ref().to_string());
        self
    }

    /// Adds a schema name to include.
    #[must_use]
    pub fn schema<S: AsRef<str>>(mut self, schema: S) -> Self {
        self.schemas.push(schema.as_ref().to_string());
        self
    }

    /// Sets the schema names to include.
    #[must_use]
    pub fn schemas<I, S>(mut self, schemas: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: ToString + AsRef<str>,
    {
        for schema in schemas {
            if !self.schemas.contains(&schema.as_ref().to_string()) {
                self = self.schema(schema);
            }
        }
        self
    }

    /// Adds a type to the denylist.
    ///
    /// # Errors
    ///
    /// Returns [`PgDatabaseBuildError::DuplicateDenylistedType`] if the type is already in the denylist.
    pub fn denylist_type<S: AsRef<str>>(mut self, ty: S) -> Result<Self, PgDatabaseBuildError> {
        let ty_str = ty.as_ref().to_string();
        if self.denylist_types.contains(&ty_str) {
            return Err(PgDatabaseBuildError::DuplicateDenylistedType(ty_str));
        }
        self.denylist_types.push(ty_str);
        Ok(self)
    }

    /// Adds multiple types to the denylist.
    ///
    /// # Errors
    ///
    /// Returns [`PgDatabaseBuildError::DuplicateDenylistedType`] if any type is already in the denylist.
    pub fn denylist_types<I, S>(mut self, types: I) -> Result<Self, PgDatabaseBuildError>
    where
        I: IntoIterator<Item = S>,
        S: ToString + AsRef<str>,
    {
        for ty in types {
            self = self.denylist_type(ty)?;
        }
        Ok(self)
    }
}

impl<'a> TryFrom<PgDieselDatabaseBuilder<'a>> for PgDieselDatabase {
    type Error = PgDatabaseBuildError;

    #[allow(clippy::needless_borrow)]
    #[allow(clippy::too_many_lines)]
    fn try_from(value: PgDieselDatabaseBuilder<'a>) -> Result<Self, Self::Error> {
        let connection = value
            .connection
            .ok_or(PgDatabaseBuildError::MissingAttribute("connection"))?;

        let table_catalog = value
            .catalog
            .ok_or(PgDatabaseBuildError::MissingAttribute("catalog"))?;

        let table_schemas = {
            if value.schemas.is_empty() {
                return Err(PgDatabaseBuildError::MissingAttribute("schemas"));
            }
            value.schemas
        };

        let mut generic_builder = GenericDBBuilder::new(table_catalog.clone(), PostgresDialect);

        for schema in Schemata::load_all(&table_catalog, &table_schemas, connection)? {
            generic_builder = generic_builder.add_schema(Arc::new(schema), ());
        }

        let cache = CatalogCache::load(connection)?;

        let roles: Vec<Arc<PgRole>> = PgRole::load_all(connection)?
            .into_iter()
            .map(Arc::new)
            .collect();

        // Functions come first: other objects reference them.
        for function in PgProc::load_all(connection)? {
            let metadata = crate::database::PgProcMetadata::new(&function, &cache);
            let schema = cache
                .namespace(function.pronamespace)
                .unwrap_or_default()
                .to_owned();
            let language = cache.language(function.prolang).map(ToOwned::to_owned);
            generic_builder = generic_builder.add_function(
                Arc::new(PgFunction::new(function, schema, language)),
                metadata,
            );
        }

        let mut tables = Vec::new();
        for table_schema in &table_schemas {
            for row in Table::load_all(connection, &table_catalog, table_schema)? {
                let partition_strategy = row.partition_strategy(connection)?;
                tables.push(Arc::new(PgTable::new(row, partition_strategy.as_deref())?));
            }
        }

        tables.sort_by(|left, right| {
            (left.schema(), left.name()).cmp(&(right.schema(), right.name()))
        });

        let mut tables_by_schema_name: HashMap<(String, String), Arc<PgTable>> = HashMap::new();
        let mut columns_by_table_column: HashMap<(String, String, String), Arc<Column>> =
            HashMap::new();
        let mut policies_by_role: HashMap<String, Vec<Arc<PgPolicy>>> = HashMap::new();

        for table in tables {
            tables_by_schema_name.insert(
                (table.schema().to_string(), table.name().to_string()),
                Arc::clone(&table),
            );

            let table_metadata = table.metadata(connection, &value.denylist_types, &cache)?;

            for column in table_metadata.column_arcs() {
                columns_by_table_column.insert(
                    (
                        table.schema().to_string(),
                        table.name().to_string(),
                        column.column_name.clone(),
                    ),
                    Arc::clone(column),
                );

                let metadata = table_metadata
                    .column_metadata(&column.column_name)
                    .expect("a column of the table carries the metadata read with it")
                    .clone();
                generic_builder = generic_builder.add_column(Arc::clone(column), metadata);
            }

            for check_constraint in table_metadata.check_constraint_arcs() {
                let metadata = check_constraint.metadata(
                    Arc::clone(&table),
                    &table_metadata,
                    generic_builder.function_arc_vec().as_slice(),
                    connection,
                )?;
                generic_builder =
                    generic_builder.add_check_constraint(Arc::clone(check_constraint), metadata);
            }

            for fk in table_metadata.foreign_key_arcs() {
                let metadata = fk.model().metadata(Arc::clone(&table), connection)?;
                generic_builder = generic_builder.add_foreign_key(Arc::clone(fk), metadata);
            }

            for index in table_metadata.unique_index_arcs() {
                generic_builder = generic_builder.add_unique_index(
                    Arc::clone(index),
                    index.metadata(Arc::clone(&table), connection)?,
                );
            }

            for index in table_metadata.index_arcs() {
                generic_builder = generic_builder.add_index(
                    Arc::clone(index),
                    index.metadata(Arc::clone(&table), connection)?,
                );
            }

            for (trigger, function_oid) in table_metadata.triggers() {
                let metadata = TriggerMetadata::new(
                    trigger.as_ref().clone(),
                    Arc::clone(&table),
                    *function_oid,
                );
                generic_builder = generic_builder.add_trigger(Arc::new(metadata), ());
            }

            for policy in table_metadata.policies() {
                let parse_expr = |sql: Option<String>| -> Option<Expr> {
                    Parser::new(&PostgreSqlDialect {})
                        .try_with_sql(&sql?)
                        .ok()?
                        .parse_expr()
                        .ok()
                };

                let (using, check) = crate::models::pg_policy_table::cached_queries::expressions(
                    policy.model(),
                    connection,
                )?;
                let using_expression = parse_expr(using);
                let check_expression = parse_expr(check);

                let mut roles = Vec::new();
                for role in crate::models::pg_policy_table::cached_queries::roles(
                    policy.model(),
                    connection,
                )? {
                    policies_by_role
                        .entry(role.rolname.clone())
                        .or_default()
                        .push(Arc::clone(policy));
                    roles.push(Owner::Ident(Ident::new(role.rolname)));
                }

                let functions = generic_builder.function_arc_vec();
                let metadata = crate::model_metadata::PolicyMetadata::new(
                    called_functions(using_expression.as_ref(), &functions),
                    called_functions(check_expression.as_ref(), &functions),
                    using_expression,
                    check_expression,
                    roles,
                );

                generic_builder = generic_builder.add_policy(Arc::clone(policy), metadata);
            }

            generic_builder = generic_builder.add_table(table, table_metadata)?;
        }

        for view in PgView::load_all(&table_schemas, connection)? {
            generic_builder =
                generic_builder.add_view(Arc::new(PgViewDefinition::plain(view)?), ());
        }
        for view in PgMatview::load_all(&table_schemas, connection)? {
            generic_builder = generic_builder
                .add_materialized_view(Arc::new(PgViewDefinition::materialized(view)?), ());
        }

        let mut role_memberships: std::collections::HashMap<u32, Vec<u32>> =
            std::collections::HashMap::new();
        let mut roles_map: std::collections::HashMap<u32, Arc<PgRole>> =
            std::collections::HashMap::new();

        for role in roles {
            let role_rc = Arc::clone(&role);

            if let Some(role_oid) = role.oid {
                roles_map.insert(role_oid, Arc::clone(&role_rc));

                let member_of_oids =
                    crate::models::pg_role::cached_queries::member_of(&role, connection)
                        .unwrap_or_default();

                role_memberships.insert(role_oid, member_of_oids);
            }
        }

        for (role_oid, role_rc) in &roles_map {
            let Some(member_of_oids) = role_memberships.get(role_oid) else {
                continue;
            };

            let member_of: Vec<Arc<PgRole>> = member_of_oids
                .iter()
                .filter_map(|oid| roles_map.get(oid).cloned())
                .collect();

            let policies = role_rc
                .rolname
                .as_ref()
                .and_then(|name| policies_by_role.remove(name))
                .unwrap_or_default();

            let metadata = crate::model_metadata::RoleMetadata::new(member_of, policies);
            generic_builder = generic_builder.add_role(Arc::clone(role_rc), metadata);
        }

        let table_grants = RoleTableGrants::load_all(&table_catalog, &table_schemas, connection)?;
        for grant in table_grants {
            let table_rc = tables_by_schema_name
                .get(&(
                    grant.table_schema.clone().unwrap_or_default(),
                    grant.table_name.clone().unwrap_or_default(),
                ))
                .cloned();

            let privilege = grant.privilege_type.as_deref().map(string_to_action);

            let grantee = grant.grantee.as_deref().map(|name| Grantee {
                grantee_type: GranteesType::None,
                name: Some(GranteeName::ObjectName(ObjectName(vec![
                    ObjectNamePart::Identifier(Ident::new(name)),
                ]))),
            });

            let metadata = RoleTableGrantsMetadata::new(privilege, grantee, table_rc);
            generic_builder = generic_builder.add_table_grant(Arc::new(grant), metadata);
        }

        let column_grants = RoleColumnGrants::load_all(&table_catalog, &table_schemas, connection)?;
        for grant in column_grants {
            let table_rc = tables_by_schema_name
                .get(&(
                    grant.table_schema.clone().unwrap_or_default(),
                    grant.table_name.clone().unwrap_or_default(),
                ))
                .cloned();

            let column_rc = table_rc.as_ref().and_then(|t| {
                columns_by_table_column
                    .get(&(
                        t.schema().to_string(),
                        t.name().to_string(),
                        grant.column_name.clone().unwrap_or_default(),
                    ))
                    .cloned()
            });

            let privilege = grant.privilege_type.as_deref().map(string_to_action);

            let grantee = grant.grantee.as_deref().map(|name| Grantee {
                grantee_type: GranteesType::None,
                name: Some(GranteeName::ObjectName(ObjectName(vec![
                    ObjectNamePart::Identifier(Ident::new(name)),
                ]))),
            });

            let metadata = RoleColumnGrantsMetadata::new(privilege, grantee, table_rc, column_rc);
            generic_builder = generic_builder.add_column_grant(Arc::new(grant), metadata);
        }

        Ok(generic_builder.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denylist_type() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        assert_eq!(builder.denylist_types, vec!["foo"]);
    }

    #[test]
    fn test_denylist_type_duplicate() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        match builder.denylist_type("foo") {
            Ok(_) => panic!("Expected error"),
            Err(e) => match e {
                PgDatabaseBuildError::DuplicateDenylistedType(t) => assert_eq!(t, "foo"),
                _ => panic!("Unexpected error: {e:?}"),
            },
        }
    }

    #[test]
    fn test_denylist_types() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_types(["foo", "bar"]).unwrap();
        assert_eq!(builder.denylist_types, vec!["foo", "bar"]);
    }

    #[test]
    fn test_denylist_types_duplicate() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        match builder.denylist_types(["bar", "foo"]) {
            Ok(_) => panic!("Expected error"),
            Err(e) => match e {
                PgDatabaseBuildError::DuplicateDenylistedType(t) => assert_eq!(t, "foo"),
                _ => panic!("Unexpected error: {e:?}"),
            },
        }
    }

    #[test]
    fn test_schema() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        assert_eq!(builder.schemas, vec!["public"]);
    }

    #[test]
    fn test_schemas() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schemas(["public", "private"]);
        assert_eq!(builder.schemas, vec!["public", "private"]);
    }

    #[test]
    fn test_schemas_deduplication() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        let builder = builder.schemas(["public", "private"]);
        // "public" is already there, so it should not be added again by schemas()
        assert_eq!(builder.schemas, vec!["public", "private"]);
    }

    #[test]
    fn test_schema_no_deduplication() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        let builder = builder.schema("public");
        // schema() does not check for duplicates
        assert_eq!(builder.schemas, vec!["public", "public"]);
    }

    #[test]
    fn test_try_from_missing_connection() {
        let builder = PgDieselDatabaseBuilder::default();
        let result: Result<PgDieselDatabase, _> = builder.try_into();
        match result {
            Err(PgDatabaseBuildError::MissingAttribute(attr)) => assert_eq!(attr, "connection"),
            _ => panic!("Unexpected result: {result:?}"),
        }
    }
}
