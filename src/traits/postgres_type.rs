//! Trait for types that can resolve their `PostgreSQL` type information.

use diesel::PgConnection;

use crate::models::PgType;

/// Trait for database objects that can resolve their `PostgreSQL` type.
pub trait PostgresType {
    /// Returns the name of the postgres type associated to the current DB
    /// object.
    ///
    /// # Errors
    ///
    /// * Returns an error if the provided database connection fails.
    fn postgres_type(&self, conn: &mut PgConnection) -> Result<PgType, diesel::result::Error>;
}
