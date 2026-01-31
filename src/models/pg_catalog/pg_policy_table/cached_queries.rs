//! Cached queries for `PgPolicyTable`.

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::PgPolicyTable;

/// Returns the roles associated with the policy.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
pub fn roles(
    policy: &PgPolicyTable,
    conn: &mut PgConnection,
) -> Result<Vec<crate::models::PgAuthid>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_authid::pg_authid;
    use diesel::ExpressionMethods;

    pg_authid::table
        .filter(pg_authid::oid.eq_any(&policy.polroles))
        .select(crate::models::PgAuthid::as_select())
        .load(conn)
}
