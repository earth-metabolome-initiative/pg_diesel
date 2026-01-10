//! Submodule providing the `TriggerMetadata` struct for a [`Triggers`](crate::models::Triggers) model.

use std::rc::Rc;

use sqlparser::ast::{TriggerEvent, TriggerObject, TriggerObjectKind, TriggerPeriod};

use crate::models::{Table, Triggers};

#[derive(Clone, Debug)]
/// Wrapper around [`Triggers`] model that holds parsed metadata.
pub struct TriggerMetadata {
    /// The underlying trigger model.
    pub model: Triggers,
    /// The table that the trigger is defined on.
    pub table: Rc<Table>,
    /// The events that fire the trigger.
    pub events: Vec<TriggerEvent>,

    /// The timing of the trigger.
    pub timing: Option<TriggerPeriod>,
    /// The orientation of the trigger.
    pub orientation: Option<TriggerObjectKind>,
    /// The OID of the function called by the trigger.
    pub function_oid: Option<u32>,
}

impl TriggerMetadata {
    /// Creates a new `TriggerMetadata` instance.
    #[must_use]
    pub fn new(model: Triggers, table: Rc<Table>, function_oid: Option<u32>) -> Self {
        let events = parse_events(&model);
        let timing = parse_timing(&model);
        let orientation = parse_orientation(&model);

        Self {
            model,
            table,
            events,
            timing,
            orientation,
            function_oid,
        }
    }

    /// Returns the table that the trigger is defined on.
    #[must_use]
    pub fn table(&self) -> &Table {
        &self.table
    }
}

/// Parses the trigger events from the model.
fn parse_events(model: &Triggers) -> Vec<TriggerEvent> {
    let mut events = Vec::new();
    if let Some(event) = &model.event_manipulation {
        match event.as_str() {
            "INSERT" => events.push(TriggerEvent::Insert),
            "UPDATE" => events.push(TriggerEvent::Update(vec![])),
            "DELETE" => events.push(TriggerEvent::Delete),
            "TRUNCATE" => events.push(TriggerEvent::Truncate),
            _ => {} // Unknown or unsupported event
        }
    }
    events
}

/// Parses the trigger timing from the model.
fn parse_timing(model: &Triggers) -> Option<TriggerPeriod> {
    model.action_timing.as_deref().and_then(|t| match t {
        "BEFORE" => Some(TriggerPeriod::Before),
        "AFTER" => Some(TriggerPeriod::After),
        "INSTEAD OF" => Some(TriggerPeriod::InsteadOf),
        _ => None,
    })
}

/// Parses the trigger orientation from the model.
fn parse_orientation(model: &Triggers) -> Option<TriggerObjectKind> {
    model.action_orientation.as_deref().and_then(|o| match o {
        "ROW" => Some(TriggerObjectKind::ForEach(TriggerObject::Row)),
        "STATEMENT" => Some(TriggerObjectKind::ForEach(TriggerObject::Statement)),
        _ => None,
    })
}
