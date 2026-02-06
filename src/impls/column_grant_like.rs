//! Implementation of `ColumnGrantLike` trait for `RoleColumnGrants`.

use std::rc::Rc;

use sqlparser::ast::{Action, Grantee};

use crate::PgDieselDatabase;
use crate::models::{Column, RoleColumnGrants, Table};
use sql_traits::traits::{
    ColumnGrantLike, ColumnLike, DatabaseLike, GrantLike, Metadata, RoleLike, TableLike,
};

/// Metadata for `RoleColumnGrants`.
///
/// Stores the parsed privilege (Action), grantee, table, and column for efficient access.
#[derive(Debug, Clone)]
pub struct RoleColumnGrantsMetadata {
    /// The parsed privilege action.
    pub privilege: Option<Action>,
    /// The parsed grantee.
    pub grantee: Option<Grantee>,
    /// The table this grant applies to.
    pub table: Option<Rc<Table>>,
    /// The column this grant applies to.
    pub column: Option<Rc<Column>>,
}

impl RoleColumnGrantsMetadata {
    /// Creates a new `RoleColumnGrantsMetadata`.
    #[must_use]
    pub fn new(
        privilege: Option<Action>,
        grantee: Option<Grantee>,
        table: Option<Rc<Table>>,
        column: Option<Rc<Column>>,
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
    pub fn table(&self) -> Option<&Table> {
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
            .and_then(|m| m.privilege())
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
            .and_then(|m| m.grantee())
            .into_iter()
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
    ) -> impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Column> {
        let column_name = self.column_name.clone();
        table.columns(database).filter(move |c| {
            // Match column name
            column_name
                .as_deref()
                .zip(Some(c.column_name()))
                .is_some_and(|(gn, cn)| gn == cn)
        })
    }

    fn table<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Table> {
        database.column_grant_metadata(self).and_then(|m| m.table())
    }
}
