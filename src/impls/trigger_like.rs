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

    fn function_name(&self) -> Option<&str> {
        // The action_statement contains something like "EXECUTE FUNCTION function_name()"
        // or "EXECUTE PROCEDURE function_name()". We parse the function name from it.
        let stmt = self.model.action_statement.as_deref()?;

        // Look for "EXECUTE FUNCTION" or "EXECUTE PROCEDURE"
        let stmt_upper = stmt.to_uppercase();
        let func_start = if let Some(pos) = stmt_upper.find("EXECUTE FUNCTION") {
            pos + "EXECUTE FUNCTION".len()
        } else if let Some(pos) = stmt_upper.find("EXECUTE PROCEDURE") {
            pos + "EXECUTE PROCEDURE".len()
        } else {
            return None;
        };

        // Extract the function name (everything before the opening parenthesis)
        let remaining = stmt[func_start..].trim();
        let name_end = remaining.find('(')?;
        let name = remaining[..name_end].trim();

        // Handle schema-qualified names (schema.function_name) - return just the function name
        if let Some(dot_pos) = name.rfind('.') {
            Some(&name[dot_pos + 1..])
        } else {
            Some(name)
        }
    }
}
