//! Metadata wrappers providing enriched information about database objects.

mod table_metadata;
pub use table_metadata::TableMetadata;
mod column_metadata;
pub use column_metadata::ColumnMetadata;
mod trigger_metadata;
pub use trigger_metadata::TriggerMetadata;
mod policy_metadata;
pub use policy_metadata::PolicyMetadata;
mod role_metadata;
pub use role_metadata::RoleMetadata;
mod pg_function;
pub use pg_function::PgFunction;
mod pg_foreign_key;
pub use pg_foreign_key::PgForeignKey;
mod pg_policy;
pub use pg_policy::PgPolicy;
mod pg_index_entry;
pub use pg_index_entry::PgIndexEntry;
mod pg_table;
pub use pg_table::{PgTable, UnknownPartitionStrategy};
mod view_definition;
pub use view_definition::{PgViewDefinition, ViewDefinitionError};
