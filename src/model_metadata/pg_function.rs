//! Submodule providing [`PgFunction`], the function a `PgDieselDatabase` holds.

use sqlparser::ast::{
    Expr, FunctionDefinitionSetParam, FunctionSetValue, Ident, ObjectName, ObjectNamePart,
};

use crate::models::PgProc;

/// A `pg_proc` row paired with the schema, language and `SET` clauses it
/// records only as OIDs or raw text.
///
/// `FunctionLike` asks for all three without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgFunction {
    /// The `pg_catalog.pg_proc` row describing the function.
    model: PgProc,
    /// The schema the function lives in, from `pg_namespace`.
    schema: String,
    /// The language the function is written in, from `pg_language`.
    language: Option<String>,
    /// The `SET` clauses attached to the function, from `proconfig`.
    configuration: Vec<FunctionDefinitionSetParam>,
}

/// Reads one `name=value` `proconfig` entry, each comma-separated part carried
/// as a bare identifier so it renders back to the text the catalog stored.
fn configuration_parameter(entry: &str) -> Option<FunctionDefinitionSetParam> {
    let (name, value) = entry.split_once('=')?;
    let values = value
        .split(',')
        .map(|part| Expr::Identifier(Ident::new(part.trim())))
        .collect::<Vec<Expr>>();
    Some(FunctionDefinitionSetParam {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new(name.trim()))]),
        value: FunctionSetValue::Values(values),
    })
}

impl PgFunction {
    /// Pairs a `pg_proc` row with the names its OIDs point at.
    #[must_use]
    pub fn new(model: PgProc, schema: String, language: Option<String>) -> Self {
        let configuration = model
            .proconfig
            .iter()
            .flatten()
            .filter_map(|entry| configuration_parameter(entry))
            .collect();
        Self {
            model,
            schema,
            language,
            configuration,
        }
    }

    /// Returns the `pg_catalog.pg_proc` row describing the function.
    #[must_use]
    pub fn model(&self) -> &PgProc {
        &self.model
    }

    /// Returns the schema the function lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the language the function is written in.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Returns the `SET` clauses attached to the function.
    #[must_use]
    pub fn configuration(&self) -> &[FunctionDefinitionSetParam] {
        &self.configuration
    }

    /// Returns the OID the catalog records the function under.
    #[must_use]
    pub fn oid(&self) -> u32 {
        self.model.oid
    }
}
