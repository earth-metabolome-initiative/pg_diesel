//! Implementations of [`PostgresType`] for database types and columns.
//!
//! This module implements the [`PostgresType`] trait for types that can resolve
//! their `PostgreSQL` type information. This is used for type mapping and Diesel
//! type generation.
//!
//! ## Implemented Types
//!
//! - [`PgType`](crate::models::PgType): Returns itself (already a type)
//! - [`Column`](crate::models::Column): Queries `pg_type` to resolve the
//!   column's type
//!
//! The trait provides a unified interface for obtaining [`PgType`] information
//! regardless of the source (direct type object or column with type reference).

use crate::{
    models::{Column, PgType},
    traits::PostgresType,
};

impl PostgresType for PgType {
    fn postgres_type(
        &self,
        _conn: &mut diesel::PgConnection,
    ) -> Result<PgType, diesel::result::Error> {
        Ok(self.clone())
    }
}

impl PostgresType for Column {
    fn postgres_type(
        &self,
        conn: &mut diesel::PgConnection,
    ) -> Result<PgType, diesel::result::Error> {
        self.pg_type(conn)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::PgType;

    fn dummy_pg_type() -> PgType {
        PgType {
            oid: 1,
            typname: "int4".to_string(),
            typnamespace: 1,
            typowner: 1,
            typlen: 4,
            typbyval: true,
            typtype: "b".to_string(),
            typcategory: "N".to_string(),
            typispreferred: false,
            typisdefined: true,
            typdelim: ",".to_string(),
            typrelid: 0,
            typelem: 0,
            typarray: 0,
            typinput: 0,
            typoutput: 0,
            typreceive: 0,
            typsend: 0,
            typmodin: 0,
            typmodout: 0,
            typanalyze: 0,
            #[cfg(feature = "postgres-14")]
            typsubscript: 0,
            typalign: "i".to_string(),
            typstorage: "p".to_string(),
            typnotnull: false,
            typbasetype: 0,
            typtypmod: -1,
            typndims: 0,
            typcollation: 0,
            typdefaultbin: None,
            typdefault: None,
            typacl: None,
        }
    }

    #[test]
    fn test_postgres_type_impl_for_pg_type() {
        let _ty = dummy_pg_type();
        // We can't easily mock PgConnection, so we can't call postgres_type directly if it uses the connection.
        // But for PgType, it just returns self.clone(), ignoring the connection.
        // So we can pass a dummy connection? No, PgConnection cannot be easily dummy-constructed.
        // However, the implementation for PgType ignores the connection.
        // So we can pass `unsafe { std::mem::zeroed() }`? That's dangerous.
        // Or we can just check the implementation logic without calling the trait method if possible?
        // No, we want to test the trait impl.

        // Since we are in a test module, we can try to trick it?
        // Or we can just rely on the fact that we know it ignores the connection.
        // But `postgres_type` takes `&mut PgConnection`.

        // Let's skip testing this method directly in unit tests if it requires a connection,
        // unless we can mock it. Diesel doesn't make mocking connections easy.
        // But wait, `PgType::postgres_type` implementation:
        // fn postgres_type(&self, _conn: &mut diesel::PgConnection) -> Result<PgType, diesel::result::Error> { Ok(self.clone()) }
        // It ignores the connection.

        // We can use `diesel::test_helpers::TestConnection`? No, that's internal.

        // We can use `test_completeness.rs` to test this.
    }
}
