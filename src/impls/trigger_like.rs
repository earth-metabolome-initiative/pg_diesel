//! Implementation of [`TriggerLike`] for [`TriggerMetadata`].

use sql_traits::traits::{DatabaseLike, Metadata, TriggerLike};
use sqlparser::ast::{TriggerEvent, TriggerObjectKind, TriggerPeriod};

use crate::{PgDieselDatabase, model_metadata::TriggerMetadata};

impl Metadata for TriggerMetadata {
    type Meta = ();
}

impl TriggerLike for TriggerMetadata {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        self.model.trigger_name.as_deref().unwrap_or("<unknown>")
    }

    fn table<'db>(&'db self, _database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        &self.table
    }

    fn events(&self) -> &[TriggerEvent] {
        &self.events
    }

    fn timing(&self) -> Option<TriggerPeriod> {
        self.timing
    }

    fn orientation(&self) -> Option<TriggerObjectKind> {
        self.orientation
    }

    fn function<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Option<&'db <Self::DB as DatabaseLike>::Function>
    where
        Self: 'db,
    {
        let oid = self.function_oid?;
        database.functions().find(|f| f.oid == oid)
    }
}
