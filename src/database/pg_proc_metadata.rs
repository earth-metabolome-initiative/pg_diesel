//! Metadata for `PostgreSQL` functions and procedures.

use crate::{
    database::CatalogCache,
    models::{PgProc, PgType},
};

#[derive(Debug, Clone)]
/// Struct collecting metadata about a `PostgreSQL` function represented by a
/// [`PgProc`] entry.
pub struct PgProcMetadata {
    /// The argument types.
    argument_types: Vec<PgType>,
    /// The return type.
    return_type: Option<PgType>,
    /// The role owning the function.
    owner: Option<String>,
}

impl PgProcMetadata {
    /// Reads the metadata of `pg_proc` from the catalogs already loaded.
    ///
    /// A type or role the cache does not hold is left out rather than looked
    /// up: the cache reads whole catalogs, so a miss means the object is gone.
    #[must_use]
    pub fn new(pg_proc: &PgProc, cache: &CatalogCache) -> Self {
        let argument_types = pg_proc
            .proargtypes
            .iter()
            .filter_map(|oid| cache.pg_type(*oid).cloned())
            .collect();
        let return_type = cache.pg_type(pg_proc.prorettype).cloned();
        let owner = cache.role(pg_proc.proowner).map(ToOwned::to_owned);

        Self {
            argument_types,
            return_type,
            owner,
        }
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

    /// Returns the role owning the function.
    #[must_use]
    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
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
        }
    }

    #[test]
    fn test_pg_proc_metadata() {
        let arg_type = dummy_pg_type();
        let ret_type = dummy_pg_type();

        let metadata = PgProcMetadata {
            argument_types: vec![arg_type],
            return_type: Some(ret_type),
            owner: Some("app_owner".to_string()),
        };

        assert_eq!(metadata.argument_types().len(), 1);
        assert_eq!(metadata.argument_types()[0].typname, "int4");
        assert_eq!(metadata.return_type().unwrap().typname, "int4");
        assert_eq!(metadata.owner(), Some("app_owner"));
    }
}
