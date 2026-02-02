//! Builder pattern for constructing a [`PgDieselDatabase`] instance.

use std::rc::Rc;

use diesel::PgConnection;
use sql_traits::{structs::generic_db::GenericDBBuilder, traits::TableLike};
use sqlparser::{ast::Ident, ast::Owner, dialect::PostgreSqlDialect, parser::Parser};

use crate::{
    PgDieselDatabase,
    model_metadata::TriggerMetadata,
    models::{PgProc, PgRole, Table},
};

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
///
/// This error type encompasses all failure modes during database metadata
/// loading:
/// - Missing required builder attributes
/// - Database query failures
/// - Invalid denylist configurations
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

        let mut generic_builder = GenericDBBuilder::new(table_catalog.clone());

        // Load all functions first as they may be referenced by other objects
        for function in PgProc::load_all(connection)? {
            let metadata = crate::database::PgProcMetadata::new(&function, connection)?;
            generic_builder = generic_builder.add_function(std::rc::Rc::new(function), metadata);
        }

        // Load all roles
        let roles: Vec<Rc<PgRole>> = PgRole::load_all(connection)?
            .into_iter()
            .map(Rc::new)
            .collect();

        let mut tables = Vec::new();
        for table_schema in &table_schemas {
            tables.extend(
                Table::load_all(connection, &table_catalog, table_schema)?
                    .into_iter()
                    .map(Rc::new),
            );
        }

        // We sort the tables by schema and name to enable efficient binary search
        // later.
        tables.sort_by_key(|table| {
            (
                table.as_ref().table_schema().unwrap_or("").to_owned(),
                table.table_name().to_owned(),
            )
        });

        // For each table, we determine all of the foreign keys and for each foreign key
        // we determine which table it references.
        for table in tables {
            let table_metadata = table.metadata(connection, &value.denylist_types)?;

            for column in table_metadata.column_rcs() {
                generic_builder = generic_builder.add_column(
                    Rc::clone(&column),
                    column.metadata(Rc::clone(&table), connection)?,
                );
            }

            for check_constraint in table_metadata.check_constraint_rcs() {
                let metadata = check_constraint.metadata(
                    Rc::clone(&table),
                    &table_metadata,
                    generic_builder.function_rc_vec().as_slice(),
                    connection,
                )?;
                generic_builder =
                    generic_builder.add_check_constraint(Rc::clone(&check_constraint), metadata);
            }

            for fk in table_metadata.foreign_key_rcs() {
                generic_builder = generic_builder
                    .add_foreign_key(Rc::clone(&fk), fk.metadata(Rc::clone(&table), connection)?);
            }

            for index in table_metadata.unique_index_rcs() {
                generic_builder = generic_builder.add_unique_index(
                    Rc::clone(&index),
                    index.metadata(Rc::clone(&table), connection)?,
                );
            }

            for (trigger, function_oid) in table_metadata.triggers() {
                let metadata = TriggerMetadata::new(
                    trigger.as_ref().clone(),
                    Rc::clone(&table),
                    *function_oid,
                );
                generic_builder = generic_builder.add_trigger(Rc::new(metadata), ());
            }

            for policy in table_metadata.policies() {
                let parse_expr = |sql: &Option<String>| -> Option<sqlparser::ast::Expr> {
                    sql.as_ref().and_then(|s| {
                        Parser::new(&PostgreSqlDialect {})
                            .try_with_sql(s)
                            .ok()?
                            .parse_expr()
                            .ok()
                    })
                };

                let using_expression = parse_expr(&policy.polqual);
                let check_expression = parse_expr(&policy.polwithcheck);

                let roles: Vec<Owner> =
                    crate::models::pg_policy_table::cached_queries::roles(policy, connection)?
                        .into_iter()
                        .map(|r| Owner::Ident(Ident::new(r.rolname)))
                        .collect();

                // Initialize Metadata with empty dependencies and table (functions will be filled later if we had logic for it)
                let metadata = crate::model_metadata::PolicyMetadata::new(
                    Rc::clone(&table),
                    Vec::new(), // using_functions
                    Vec::new(), // check_functions
                    using_expression,
                    check_expression,
                    roles,
                );

                generic_builder = generic_builder.add_policy(Rc::clone(policy), metadata);
            }

            generic_builder = generic_builder.add_table(table, table_metadata);
        }

        // Collect all roles' membership data and prepare role Rcs
        let mut role_memberships: std::collections::HashMap<u32, Vec<u32>> =
            std::collections::HashMap::new();
        let mut roles_map: std::collections::HashMap<u32, Rc<PgRole>> =
            std::collections::HashMap::new();

        // First, create Rc for all roles and query their memberships
        for role in roles {
            let role_rc = Rc::clone(&role);

            if let Some(role_oid) = role.oid {
                roles_map.insert(role_oid, Rc::clone(&role_rc));

                // Query pg_auth_members for this role's memberships
                let member_of_oids =
                    crate::models::pg_role::cached_queries::member_of(&role, connection)
                        .unwrap_or_default();

                role_memberships.insert(role_oid, member_of_oids);
            }
        }

        // Now add roles with their metadata
        // Note: We cannot populate policies here because policies reference roles by name,
        // and we'd need to query the already-built database. For now, we leave policies empty.
        for (role_oid, role_rc) in &roles_map {
            let Some(member_of_oids) = role_memberships.get(role_oid) else {
                continue;
            };

            // Find the actual role Rcs from our map
            let member_of: Vec<Rc<PgRole>> = member_of_oids
                .iter()
                .filter_map(|oid| roles_map.get(oid).cloned())
                .collect();

            // For now, we don't populate policies (would require post-build processing)
            let policies = Vec::new();

            let metadata = crate::model_metadata::RoleMetadata::new(member_of, policies);
            generic_builder = generic_builder.add_role(Rc::clone(role_rc), metadata);
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
