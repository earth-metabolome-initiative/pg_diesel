//! The `DatabaseLike` catalog a `PostgreSQL` connection is read into.

use sql_traits::structs::{GenericDB, SchemaProfile};

use crate::dialect::PostgresDialect;
use crate::model_metadata::{
    PgForeignKey, PgFunction, PgIndexEntry, PgPolicy, PgTable, PgViewDefinition, TriggerMetadata,
};
use crate::models::{CheckConstraint, Column, PgRole, RoleColumnGrants, RoleTableGrants, Schemata};
mod key_column_usage_metadata;
pub use key_column_usage_metadata::KeyColumnUsageMetadata;
mod pg_proc_metadata;
pub use pg_proc_metadata::PgProcMetadata;
mod catalog_cache;
pub use catalog_cache::CatalogCache;

mod builder;
pub use builder::{PgDatabaseBuildError, PgDieselDatabaseBuilder};

/// The kinds of schema object a database read through `pg_catalog` holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgDieselProfile;

impl SchemaProfile for PgDieselProfile {
    type CheckConstraint = CheckConstraint;
    type Column = Column;
    type ColumnGrant = RoleColumnGrants;
    type Dialect = PostgresDialect;
    type ForeignKey = PgForeignKey;
    type Function = PgFunction;
    type Index = PgIndexEntry;
    /// A catalog read holds no statement-by-statement state: the whole
    /// database is loaded in one pass and never resumed.
    type Ingestion = ();
    type MaterializedView = PgViewDefinition;
    type Policy = PgPolicy;
    type Role = PgRole;
    type Schema = Schemata;
    type Table = PgTable;
    type TableGrant = RoleTableGrants;
    type Trigger = TriggerMetadata;
    type UniqueIndex = PgIndexEntry;
    type View = PgViewDefinition;

    fn default_ingestion(_dialect: &Self::Dialect) -> Self::Ingestion {}
}

/// Type alias representing a `PostgreSQL` database with loaded metadata.
pub type PgDieselDatabase = GenericDB<PgDieselProfile>;
