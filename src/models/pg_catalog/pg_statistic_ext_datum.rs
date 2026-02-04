//! Submodule providing the `PgStatisticExtDatum` struct representing a row of
//! the `pg_statistic_ext_data` table in `PostgreSQL`.

use diesel::{Identifiable, Queryable, QueryableByName, Selectable};

#[cfg(feature = "postgres-14")]
/// Represents a row from the `pg_statistic_ext_data` table (`PostgreSQL` 14).
///
/// The `pg_statistic_ext_data` table holds data for extended planner statistics
/// objects, containing the actual statistical values.
///
/// For more information, see the [PostgreSQL documentation](https://www.postgresql.org/docs/current/catalog-pg-statistic-ext-data.html).
#[derive(
    Identifiable, Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_statistic_ext_data::pg_statistic_ext_data)]
#[diesel(primary_key(stxoid))]
pub struct PgStatisticExtDatum {
    /// Statistics object OID.
    pub stxoid: u32,
    /// N-distinct counts.
    pub stxdndistinct: Option<Vec<u8>>,
    /// Functional dependency statistics.
    pub stxddependencies: Option<Vec<u8>>,
    /// MCV list statistics.
    pub stxdmcv: Option<Vec<u8>>,
    /// Statistics for expressions.
    pub stxdexpr: Option<Vec<u8>>,
}

#[cfg(not(feature = "postgres-14"))]
/// Represents a row from the `pg_statistic_ext_data` table (`PostgreSQL` 15+).
///
/// The `pg_statistic_ext_data` table holds data for extended planner statistics
/// objects, containing the actual statistical values.
///
/// For more information, see the [PostgreSQL documentation](https://www.postgresql.org/docs/current/catalog-pg-statistic-ext-data.html).
#[derive(
    Identifiable, Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_statistic_ext_data::pg_statistic_ext_data)]
#[diesel(primary_key(stxoid, stxdinherit))]
pub struct PgStatisticExtDatum {
    /// Statistics object OID.
    pub stxoid: u32,
    /// Inherited stats flag.
    pub stxdinherit: bool,
    /// N-distinct counts.
    pub stxdndistinct: Option<Vec<u8>>,
    /// Functional dependency statistics.
    pub stxddependencies: Option<Vec<u8>>,
    /// MCV list statistics.
    pub stxdmcv: Option<Vec<u8>>,
    /// Statistics for expressions.
    pub stxdexpr: Option<Vec<u8>>,
}
