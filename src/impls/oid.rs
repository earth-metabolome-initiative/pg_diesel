//! Implementations of [`HasOid`] for `PostgreSQL` catalog types.
//!
//! This module implements the [`HasOid`] trait for various model structs that
//! have `PostgreSQL` Object Identifiers (OIDs). OIDs are unique identifiers for
//! database objects in the system catalogs.
//!
//! ## Implemented Types
//!
//! The trait is implemented for:
//! - [`PgIndex`](crate::models::PgIndex): Returns `indexrelid`
//! - [`PgConstraint`](crate::models::PgConstraint): Returns `oid`
//! - [`PgExtension`](crate::models::PgExtension): Returns `oid`
//! - [`PgOperator`](crate::models::PgOperator): Returns `oid`
//! - [`PgType`](crate::models::PgType): Returns `oid`
//! - [`PgProc`](crate::models::PgProc): Returns `oid`
//! - [`PgDescription`](crate::models::PgDescription): Returns `objoid` (the OID
//!   of the described object)
//!
//! These implementations are used by caching infrastructure to efficiently key
//! cached values.

use crate::traits::HasOid;

impl HasOid for crate::models::PgIndex {
    fn oid(&self) -> u32 {
        self.indexrelid
    }
}

impl HasOid for crate::models::PgConstraint {
    fn oid(&self) -> u32 {
        self.oid
    }
}

impl HasOid for crate::models::PgExtension {
    fn oid(&self) -> u32 {
        self.oid
    }
}

impl HasOid for crate::models::PgOperator {
    fn oid(&self) -> u32 {
        self.oid
    }
}

impl HasOid for crate::models::PgType {
    fn oid(&self) -> u32 {
        self.oid
    }
}

impl HasOid for crate::models::PgProc {
    fn oid(&self) -> u32 {
        self.oid
    }
}

impl HasOid for crate::models::PgDescription {
    fn oid(&self) -> u32 {
        self.objoid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        PgConstraint, PgDescription, PgExtension, PgIndex, PgOperator, PgProc, PgType,
    };

    fn dummy_pg_index() -> PgIndex {
        PgIndex {
            indexrelid: 1,
            indrelid: 0,
            indnatts: 0,
            indnkeyatts: 0,
            indisunique: false,
            #[cfg(any(
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            indnullsnotdistinct: false,
            indisprimary: false,
            indisexclusion: false,
            indimmediate: false,
            indisclustered: false,
            indisvalid: false,
            indcheckxmin: false,
            indisready: false,
            indislive: false,
            indisreplident: false,
            indkey: vec![],
            indcollation: vec![],
            indclass: vec![],
            indoption: vec![],
            indexprs: None,
            indpred: None,
        }
    }

    fn dummy_pg_constraint() -> PgConstraint {
        PgConstraint {
            oid: 2,
            conname: String::new(),
            connamespace: 0,
            contype: String::new(),
            condeferrable: false,
            condeferred: false,
            convalidated: false,
            #[cfg(feature = "postgres-18")]
            conenforced: false,
            conrelid: 0,
            contypid: 0,
            conindid: 0,
            conparentid: 0,
            confrelid: 0,
            confupdtype: String::new(),
            confdeltype: String::new(),
            confmatchtype: String::new(),
            conislocal: false,
            coninhcount: 0,
            connoinherit: false,
            conkey: None,
            #[cfg(feature = "postgres-18")]
            conperiod: false,
            confkey: None,
            conpfeqop: None,
            conppeqop: None,
            conffeqop: None,
            conexclop: None,
            #[cfg(any(
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            confdelsetcols: None,
            conbin: None,
        }
    }

    fn dummy_pg_operator() -> PgOperator {
        PgOperator {
            oid: 4,
            oprname: String::new(),
            oprnamespace: 0,
            oprowner: 0,
            oprkind: String::new(),
            oprcanmerge: false,
            oprcanhash: false,
            oprleft: 0,
            oprright: 0,
            oprresult: 0,
            oprcom: 0,
            oprnegate: 0,
            oprcode: 0,
            oprrest: 0,
            oprjoin: 0,
        }
    }

    fn dummy_pg_type() -> PgType {
        PgType {
            oid: 5,
            typname: String::new(),
            typnamespace: 0,
            typowner: 0,
            typlen: 0,
            typbyval: false,
            typtype: String::new(),
            typcategory: String::new(),
            typispreferred: false,
            typisdefined: false,
            typdelim: String::new(),
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
            typalign: String::new(),
            typstorage: String::new(),
            typnotnull: false,
            typbasetype: 0,
            typtypmod: 0,
            typndims: 0,
            typcollation: 0,
            typdefaultbin: None,
            typdefault: None,
            typacl: None,
        }
    }

    fn dummy_pg_proc() -> PgProc {
        PgProc {
            oid: 6,
            proname: String::new(),
            pronamespace: 0,
            proowner: 0,
            prolang: 0,
            procost: 0.0,
            prorows: 0.0,
            provariadic: 0,
            prosupport: 0,
            prokind: String::new(),
            prosecdef: false,
            proleakproof: false,
            proisstrict: false,
            proretset: false,
            provolatile: String::new(),
            proparallel: String::new(),
            pronargs: 0,
            pronargdefaults: 0,
            prorettype: 0,
            proargtypes: vec![],
            proallargtypes: None,
            proargmodes: None,
            proargnames: None,
            proargdefaults: None,
            protrftypes: None,
            prosrc: String::new(),
            probin: None,
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            prosqlbody: None,
            proconfig: None,
            proacl: None,
        }
    }

    #[test]
    fn test_oid_impls() {
        let index = dummy_pg_index();
        assert_eq!(index.oid(), 1);

        let constraint = dummy_pg_constraint();
        assert_eq!(constraint.oid(), 2);

        let extension = PgExtension {
            oid: 3,
            extname: String::new(),
            extowner: 0,
            extnamespace: 0,
            extrelocatable: false,
            extversion: String::new(),
            extconfig: None,
            extcondition: None,
        };
        assert_eq!(extension.oid(), 3);

        let operator = dummy_pg_operator();
        assert_eq!(operator.oid(), 4);

        let ty = dummy_pg_type();
        assert_eq!(ty.oid(), 5);

        let proc = dummy_pg_proc();
        assert_eq!(proc.oid(), 6);

        let desc = PgDescription {
            objoid: 7,
            classoid: 0,
            objsubid: 0,
            description: String::new(),
        };
        assert_eq!(desc.oid(), 7);
    }
}
