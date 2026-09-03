//! `PolicyMetadata` struct.

use crate::model_metadata::PgFunction;
use sqlparser::ast::{Expr, Owner};
use std::sync::Arc;

#[derive(Debug, Clone)]
/// Metadata for a policy.
pub struct PolicyMetadata {
    /// Functions used in USING expression.
    pub using_functions: Vec<Arc<PgFunction>>,
    /// Functions used in WITH CHECK expression.
    pub check_functions: Vec<Arc<PgFunction>>,
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
        using_functions: Vec<Arc<PgFunction>>,
        check_functions: Vec<Arc<PgFunction>>,
        using_expression: Option<Expr>,
        check_expression: Option<Expr>,
        roles: Vec<Owner>,
    ) -> Self {
        Self {
            using_functions,
            check_functions,
            using_expression,
            check_expression,
            roles,
        }
    }

    /// Returns using functions.
    pub fn using_functions(&self) -> impl Iterator<Item = &Arc<PgFunction>> {
        self.using_functions.iter()
    }

    /// Returns check functions.
    pub fn check_functions(&self) -> impl Iterator<Item = &Arc<PgFunction>> {
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
