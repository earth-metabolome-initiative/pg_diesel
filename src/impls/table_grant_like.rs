//! Implementation of `TableGrantLike` trait for `RoleTableGrants`.

use std::rc::Rc;

use sqlparser::ast::{Action, Grantee};

use crate::PgDieselDatabase;
use crate::models::{RoleTableGrants, Table};
use sql_traits::traits::{DatabaseLike, GrantLike, Metadata, RoleLike, TableGrantLike, TableLike};

/// Metadata for `RoleTableGrants`.
///
/// Stores the parsed privilege (Action) and grantee for efficient access.
#[derive(Debug, Clone)]
pub struct RoleTableGrantsMetadata {
    /// The parsed privilege action.
    pub privilege: Option<Action>,
    /// The parsed grantee.
    pub grantee: Option<Grantee>,
    /// The table this grant applies to.
    pub table: Option<Rc<Table>>,
}

impl RoleTableGrantsMetadata {
    /// Creates a new `RoleTableGrantsMetadata`.
    #[must_use]
    pub fn new(
        privilege: Option<Action>,
        grantee: Option<Grantee>,
        table: Option<Rc<Table>>,
    ) -> Self {
        Self {
            privilege,
            grantee,
            table,
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
}

impl Metadata for RoleTableGrants {
    type Meta = RoleTableGrantsMetadata;
}

impl GrantLike for RoleTableGrants {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        database
            .table_grant_metadata(self)
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
            .table_grant_metadata(self)
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

impl TableGrantLike for RoleTableGrants {
    fn tables<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Table> {
        database.tables().filter(move |t| {
            let schema_match = match (self.table_schema.as_deref(), t.table_schema()) {
                (Some(gs), Some(ts)) => gs == ts,
                (None, None) => true,
                _ => false,
            };
            let name_match = self
                .table_name
                .as_deref()
                .is_some_and(|n| n == t.table_name());
            schema_match && name_match
        })
    }

    fn applies_to_table(
        &self,
        table: &<Self::DB as DatabaseLike>::Table,
        _database: &Self::DB,
    ) -> bool {
        let schema_match = match (self.table_schema.as_deref(), table.table_schema()) {
            (Some(gs), Some(ts)) => gs == ts,
            (None, None) => true,
            _ => false,
        };
        let name_match = self
            .table_name
            .as_deref()
            .is_some_and(|n| n == table.table_name());
        schema_match && name_match
    }
}

/// Convert a privilege type string to an `Action`.
#[must_use]
pub fn string_to_action(privilege: &str) -> Action {
    match privilege.to_uppercase().as_str() {
        "SELECT" => Action::Select { columns: None },
        "INSERT" => Action::Insert { columns: None },
        "UPDATE" => Action::Update { columns: None },
        "DELETE" => Action::Delete,
        "TRUNCATE" => Action::Truncate,
        "REFERENCES" => Action::References { columns: None },
        "TRIGGER" => Action::Trigger,
        _ => Action::Usage,
    }
}
