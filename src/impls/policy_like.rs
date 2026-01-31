//! `PolicyLike` implementation.

use sql_traits::traits::{DocumentationMetadata, Metadata, PolicyLike};
use sqlparser::ast::{CreatePolicyCommand, Owner};

use crate::database::PgDieselDatabase;
use crate::model_metadata::PolicyMetadata;
use crate::models::PgPolicyTable;

impl Metadata for PgPolicyTable {
    type Meta = PolicyMetadata;
}

impl DocumentationMetadata for PgPolicyTable {
    type Documentation = ();
}

impl PolicyLike for PgPolicyTable {
    type DB = PgDieselDatabase;

    fn table<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> &'db <Self::DB as sql_traits::prelude::DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .policy_metadata(self)
            .expect("Policy must exist in database")
            .table()
    }

    fn roles<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Owner>
    where
        Self: 'db,
    {
        database
            .policy_metadata(self)
            .expect("Policy metadata")
            .roles()
    }

    fn using_expression<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Option<&'db sqlparser::ast::Expr>
    where
        Self: 'db,
    {
        database
            .policy_metadata(self)
            .expect("Policy metadata")
            .using_expression()
    }

    fn check_expression<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Option<&'db sqlparser::ast::Expr>
    where
        Self: 'db,
    {
        database
            .policy_metadata(self)
            .expect("Policy metadata")
            .check_expression()
    }

    fn command(&self) -> CreatePolicyCommand {
        match self.polcmd.as_str() {
            "r" => CreatePolicyCommand::Select,
            "a" => CreatePolicyCommand::Insert,
            "w" => CreatePolicyCommand::Update,
            "d" => CreatePolicyCommand::Delete,
            "*" => CreatePolicyCommand::All,
            _ => panic!("Unknown policy command: {}", self.polcmd),
        }
    }

    fn name(&self) -> &str {
        &self.polname
    }

    fn using_functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = &'db <Self::DB as sql_traits::prelude::DatabaseLike>::Function> {
        database
            .policy_metadata(self)
            .expect("Policy")
            .using_functions()
            .map(std::convert::AsRef::as_ref)
    }

    fn check_functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = &'db <Self::DB as sql_traits::prelude::DatabaseLike>::Function> {
        database
            .policy_metadata(self)
            .expect("Policy")
            .check_functions()
            .map(std::convert::AsRef::as_ref)
    }
}
