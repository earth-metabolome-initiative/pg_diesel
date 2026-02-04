//! Metadata for `PostgreSQL` functions and procedures.
//!
//! This module provides [`PgProcMetadata`], which encapsulates type information
//! for a `PostgreSQL` function or procedure, including:
//! - Argument types (from `pg_proc.proargtypes`)
//! - Return type (from `pg_proc.prorettype`)
//!
//! This metadata is used by the [`PgDieselDatabase`](crate::database::PgDieselDatabase) to
//! provide function introspection through the `sql_traits` trait system.

use diesel::PgConnection;

use crate::models::{PgProc, PgType};

#[derive(Debug, Clone)]
/// Struct collecting metadata about a `PostgreSQL` function represented by a
/// [`PgProc`] entry.
pub struct PgProcMetadata {
    /// The argument types.
    argument_types: Vec<PgType>,
    /// The return type.
    return_type: Option<PgType>,
}

impl PgProcMetadata {
    /// Creates a new `PgProcMetadata` instance from a `PgProc` and database
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `pg_proc` - The `PostgreSQL` function to get metadata for.
    /// * `conn` - A mutable reference to a `PostgreSQL` connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the type information cannot be retrieved from the
    /// database.
    pub fn new(pg_proc: &PgProc, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        let argument_types = pg_proc.argument_types(conn)?;

        let return_type = if pg_proc.prorettype == 0 {
            None
        } else {
            Some(pg_proc.return_type(conn)?)
        };

        Ok(Self {
            argument_types,
            return_type,
        })
    }

    /// Returns the argument types.
    #[must_use]
    pub fn argument_types(&self) -> &[PgType] {
        &self.argument_types
    }

    /// Returns the return type.
    #[must_use]
    pub fn return_type(&self) -> Option<&PgType> {
        self.return_type.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
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
    fn test_pg_proc_metadata() {
        let arg_type = dummy_pg_type();
        let ret_type = dummy_pg_type();

        let metadata = PgProcMetadata {
            argument_types: vec![arg_type],
            return_type: Some(ret_type),
        };

        assert_eq!(metadata.argument_types().len(), 1);
        assert_eq!(metadata.argument_types()[0].typname, "int4");
        assert_eq!(metadata.return_type().unwrap().typname, "int4");
    }
}
