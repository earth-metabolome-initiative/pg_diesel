//! Submodule providing the `PgDbRoleSetting` struct representing a row of the
//! `pg_db_role_setting` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_db_role_setting` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_db_role_setting::pg_db_role_setting)]
pub struct PgDbRoleSetting {
    /// OID of the database this setting applies to (0 = all databases).
    pub setdatabase: u32,
    /// OID of the role this setting applies to (0 = all roles).
    pub setrole: u32,
    /// Array of configuration parameter settings.
    pub setconfig: Option<Vec<String>>,
}
