//! Implementation of `ColumnGrantLike` trait for `RoleColumnGrants`.

use std::sync::Arc;

use sql_traits::{
    errors::LookupError,
    structs::TargetName,
    traits::{
        ColumnGrantLike, ColumnLike, DatabaseLike, GrantLike, Metadata, RoleLike, TableLike,
        ViewLike, grant::GrantRelation,
    },
};
use sqlparser::ast::{Action, Grantee};

use crate::{
    PgDieselDatabase,
    model_metadata::PgTable,
    models::{Column, RoleColumnGrants},
};

/// Metadata for `RoleColumnGrants`.
#[derive(Debug, Clone)]
pub struct RoleColumnGrantsMetadata {
    /// The parsed privilege action.
    pub privilege: Option<Action>,
    /// The parsed grantee.
    pub grantee: Option<Grantee>,
    /// The table this grant applies to.
    pub table: Option<Arc<PgTable>>,
    /// The column this grant applies to.
    pub column: Option<Arc<Column>>,
}

impl RoleColumnGrantsMetadata {
    /// Creates a new `RoleColumnGrantsMetadata`.
    #[must_use]
    pub fn new(
        privilege: Option<Action>,
        grantee: Option<Grantee>,
        table: Option<Arc<PgTable>>,
        column: Option<Arc<Column>>,
    ) -> Self {
        Self {
            privilege,
            grantee,
            table,
            column,
        }
    }

    /// Returns the privilege action.
    #[must_use]
    pub fn privilege(&self) -> Option<&Action> {
        self.privilege.as_ref()
    }

    /// Returns the grantee.
    #[must_use]
    pub fn grantee(&self) -> Option<&Grantee> {
        self.grantee.as_ref()
    }

    /// Returns the table.
    #[must_use]
    pub fn table(&self) -> Option<&PgTable> {
        self.table.as_deref()
    }

    /// Returns the column.
    #[must_use]
    pub fn column(&self) -> Option<&Column> {
        self.column.as_deref()
    }
}

impl Metadata for RoleColumnGrants {
    type Meta = RoleColumnGrantsMetadata;
}

impl GrantLike for RoleColumnGrants {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::privilege)
            .into_iter()
    }

    fn is_all_privileges(&self) -> bool {
        self.privilege_type.as_deref().is_some_and(|p| {
            p.eq_ignore_ascii_case("ALL") || p.eq_ignore_ascii_case("ALL PRIVILEGES")
        })
    }

    fn grantees<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Grantee>
    where
        Self: 'db,
    {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::grantee)
            .into_iter()
    }

    fn applies_to_public(&self) -> bool {
        self.grantee
            .as_deref()
            .is_some_and(|grantee| grantee.eq_ignore_ascii_case("PUBLIC"))
    }

    fn target_table_names(&self) -> impl Iterator<Item = TargetName<'_>> {
        self.table_name
            .as_deref()
            .map(|name| {
                let target = TargetName::new(name, true);
                match self.table_schema.as_deref() {
                    Some(schema) => target.with_schema(schema, true),
                    None => target,
                }
            })
            .into_iter()
    }

    fn target_schema_names(&self) -> impl Iterator<Item = TargetName<'_>> {
        core::iter::empty()
    }

    fn with_grant_option(&self) -> bool {
        self.is_grantable
            .as_deref()
            .is_some_and(|g| g.eq_ignore_ascii_case("YES"))
    }

    fn granted_by<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Role> {
        let grantor_name = self.grantor.as_deref()?;
        database.roles().find(|r| r.name() == grantor_name)
    }

    fn applies_to_role(&self, role: &<Self::DB as DatabaseLike>::Role) -> bool {
        self.grantee.as_deref().is_some_and(|g| g == role.name())
    }
}

impl ColumnGrantLike for RoleColumnGrants {
    fn columns<'a>(
        &'a self,
        table: &'a <Self::DB as DatabaseLike>::Table,
        database: &'a Self::DB,
    ) -> Result<impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Column>, LookupError> {
        Ok(table
            .columns(database)?
            .filter(move |column| self.column_name.as_deref() == Some(column.column_name())))
    }

    fn table<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Table> {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::table)
    }

    fn relation<'a>(&'a self, database: &'a Self::DB) -> Option<GrantRelation<'a, Self::DB>> {
        if let Some(table) = self.table(database) {
            return Some(GrantRelation::Table(table));
        }
        let names = |schema: Option<&str>, name: &str| {
            self.table_schema.as_deref() == schema && self.table_name.as_deref() == Some(name)
        };
        if let Some(view) = database
            .views()
            .find(|v| names(v.view_schema(), v.view_name()))
        {
            return Some(GrantRelation::View(view));
        }
        database
            .materialized_views()
            .find(|v| names(v.view_schema(), v.view_name()))
            .map(GrantRelation::MaterializedView)
    }
}
