//! Submodule providing [`PgTable`], the base table a `PgDieselDatabase` holds.

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sql_traits::traits::PartitionStrategy;

use crate::models::Table;

/// A base table row paired with the partitioning strategy it omits.
///
/// `TableLike::partition_strategy` takes no database, so the answer has to
/// live on the value itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgTable {
    /// The `information_schema.tables` row describing the table.
    model: Table,
    /// How the table routes rows to its partitions, when it is partitioned.
    partition_strategy: Option<PartitionStrategy>,
}

/// A `pg_partitioned_table.partstrat` value this crate does not know.
#[derive(Debug, thiserror::Error)]
#[error("`{0}` is not a partitioning strategy PostgreSQL documents")]
pub struct UnknownPartitionStrategy(String);

/// Ranks a strategy, since [`PartitionStrategy`] derives neither [`Ord`] nor
/// [`Hash`] and `TableLike` requires both.
fn strategy_rank(strategy: Option<PartitionStrategy>) -> u8 {
    match strategy {
        None => 0,
        Some(PartitionStrategy::Range) => 1,
        Some(PartitionStrategy::List) => 2,
        Some(PartitionStrategy::Hash) => 3,
    }
}

impl Hash for PgTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.model.hash(state);
        strategy_rank(self.partition_strategy).hash(state);
    }
}

impl PartialOrd for PgTable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PgTable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.model.cmp(&other.model).then_with(|| {
            strategy_rank(self.partition_strategy).cmp(&strategy_rank(other.partition_strategy))
        })
    }
}

impl std::fmt::Display for PgTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.model.fmt(f)
    }
}

impl AsRef<Table> for PgTable {
    fn as_ref(&self) -> &Table {
        &self.model
    }
}

impl PgTable {
    /// Pairs an `information_schema.tables` row with its partitioning strategy.
    ///
    /// `partstrat` is the `pg_catalog.pg_partitioned_table.partstrat` value
    /// recorded for the table, or [`None`] when the table is not partitioned.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownPartitionStrategy`] when `partstrat` is neither `l`,
    /// `r` nor `h`, rather than reading an unrecognised strategy as no
    /// partitioning at all.
    pub fn new(model: Table, partstrat: Option<&str>) -> Result<Self, UnknownPartitionStrategy> {
        let partition_strategy = match partstrat {
            None => None,
            Some("l") => Some(PartitionStrategy::List),
            Some("r") => Some(PartitionStrategy::Range),
            Some("h") => Some(PartitionStrategy::Hash),
            Some(other) => return Err(UnknownPartitionStrategy(other.to_owned())),
        };
        Ok(Self {
            model,
            partition_strategy,
        })
    }

    /// Returns the `information_schema.tables` row describing the table.
    #[must_use]
    pub fn model(&self) -> &Table {
        &self.model
    }

    /// Returns the catalog the table belongs to.
    #[must_use]
    pub fn catalog(&self) -> &str {
        &self.model.table_catalog
    }

    /// Returns the schema the table belongs to.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.model.table_schema
    }

    /// Returns the name of the table.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.model.table_name
    }

    /// Returns how the table routes rows to its partitions.
    #[must_use]
    pub fn partition_strategy(&self) -> Option<PartitionStrategy> {
        self.partition_strategy
    }

    /// Initializes and returns the metadata for the table.
    ///
    /// # Errors
    ///
    /// * If the metadata cannot be loaded from the database.
    pub fn metadata(
        self: &std::sync::Arc<Self>,
        conn: &mut diesel::PgConnection,
        denylist_types: &[String],
        cache: &crate::database::CatalogCache,
    ) -> Result<crate::model_metadata::TableMetadata, diesel::result::Error> {
        self.model.metadata(self, conn, denylist_types, cache)
    }
}
