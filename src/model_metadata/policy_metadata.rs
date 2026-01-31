//! `PolicyMetadata` struct.

use crate::models::{PgProc, Table};
use sqlparser::ast::{Expr, Owner};
use std::rc::Rc;

#[derive(Debug, Clone)]
/// Metadata for a policy.
pub struct PolicyMetadata {
    /// The table the policy belongs to.
    pub table: Rc<Table>,
    /// Functions used in USING expression.
    pub using_functions: Vec<Rc<PgProc>>,
    /// Functions used in WITH CHECK expression.
    pub check_functions: Vec<Rc<PgProc>>,
    /// Parsed USING expression.
    pub using_expression: Option<Expr>,
    /// Parsed WITH CHECK expression.
    pub check_expression: Option<Expr>,
    /// Roles the policy applies to.
    pub roles: Vec<Owner>,
}

impl PolicyMetadata {
    /// Creates a new `PolicyMetadata`.
    #[must_use]
    pub fn new(
        table: Rc<Table>,
        using_functions: Vec<Rc<PgProc>>,
        check_functions: Vec<Rc<PgProc>>,
        using_expression: Option<Expr>,
        check_expression: Option<Expr>,
        roles: Vec<Owner>,
    ) -> Self {
        Self {
            table,
            using_functions,
            check_functions,
            using_expression,
            check_expression,
            roles,
        }
    }

    /// Returns the table.
    #[must_use]
    pub fn table(&self) -> &Table {
        &self.table
    }

    /// Returns using functions.
    pub fn using_functions(&self) -> impl Iterator<Item = &Rc<PgProc>> {
        self.using_functions.iter()
    }

    /// Returns check functions.
    pub fn check_functions(&self) -> impl Iterator<Item = &Rc<PgProc>> {
        self.check_functions.iter()
    }

    /// Returns the using expression.
    #[must_use]
    pub fn using_expression(&self) -> Option<&Expr> {
        self.using_expression.as_ref()
    }

    /// Returns the check expression.
    #[must_use]
    pub fn check_expression(&self) -> Option<&Expr> {
        self.check_expression.as_ref()
    }

    /// Returns the roles.
    pub fn roles(&self) -> impl Iterator<Item = &Owner> {
        self.roles.iter()
    }
}
