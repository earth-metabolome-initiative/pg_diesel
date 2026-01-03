//! Submodule testing that all relevant postgres tables exist in the schema.

use std::path::PathBuf;
use std::rc::Rc;

use diesel::{Connection, PgConnection};
use pg_diesel::database::{PgDieselDatabase, PgDieselDatabaseBuilder};
use pg_diesel::models::{PgExtension, PgProc, Table};
use pg_diesel::traits::PostgresType;
use sql_traits::traits::{
    CheckConstraintLike, ColumnLike, ForeignKeyLike, TableLike, UniqueIndexLike,
    database::DatabaseLike,
};
use testcontainers::ImageExt;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, TestcontainersError, core::WaitFor};

/// Setup a docker container with a postgres database.
///
/// # Arguments
///
/// * `database_port` - The port of the database.
/// * `database_name` - The name of the database.
///
/// # Panics
///
/// * If the container cannot be started.
///
/// # Errors
///
/// * If there is an error starting the container.
async fn reference_docker(
    database_port: u16,
    database_name: &str,
) -> Result<ContainerAsync<GenericImage>, TestcontainersError> {
    GenericImage::new("postgres", "17.4")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
        .with_env_var("POSTGRES_USER", "user")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", database_name)
        .with_mapped_port(database_port, 5432_u16.tcp())
        .start()
        .await
}

/// Establish a connection to a postgres database.
///
/// # Arguments
///
/// * `database_port` - The port of the database.
/// * `database_name` - The name of the database.
///
/// # Errors
///
/// * If the connection cannot be established.
fn establish_connection_to_postgres<C: Connection>(
    database_port: u16,
    database_name: &str,
) -> Result<C, diesel::ConnectionError> {
    let database_url =
        format!("postgres://user:password@localhost:{database_port}/{database_name}",);

    let mut number_of_attempts = 0;

    while let Err(e) = C::establish(&database_url) {
        eprintln!("Failed to establish connection: {e:?}");
        std::thread::sleep(std::time::Duration::from_secs(1));
        if number_of_attempts > 10 {
            eprintln!("Failed to establish connection after 10 attempts");
            std::process::exit(1);
        }
        number_of_attempts += 1;
    }

    C::establish(&database_url)
}

#[tokio::test]
/// Test retrieval of extensions from a column
async fn test_schema_completeness() {
    let database_name = "test_schema_completeness";
    let port = 35433;
    let docker = reference_docker(port, database_name)
        .await
        .expect("Failed to start docker container");
    let mut conn: PgConnection = establish_connection_to_postgres(port, database_name)
        .expect("Failed to establish connection to database");

    let db: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut conn)
        .catalog(database_name)
        .denylist_types([
            "anyarray",
            "pg_ndistinct",
            "pg_dependencies",
            "pg_mcv_list",
            "_pg_statistic",
        ])
        .unwrap()
        .schemas(["pg_toast", "pg_catalog", "information_schema", "public"])
        .try_into()
        .expect("Failed to build database");

    let crate_root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for table in db.table_dag() {
        if table.table_name().starts_with('_') {
            // Skip internal tables
            continue;
        }

        let expected_schema_path = crate_root_path
            .join("src")
            .join("schema")
            .join(table.table_schema().unwrap())
            .join(format!("{}.rs", table.table_name()));

        assert!(
            expected_schema_path.exists(),
            "Table `{}` not found in src/schema/",
            expected_schema_path.display(),
        );

        // We read the content of the expected schema file
        let expected_schema_content = std::fs::read_to_string(&expected_schema_path)
            .expect("Unable to read expected schema file");

        // We check that all columns are found in the expected schema file
        for column in table.columns(&db) {
            assert!(
                expected_schema_content.contains(column.column_name()),
                "Column `{}.{}` not found in expected schema file",
                table.table_name(),
                column.column_name(),
            );
        }
    }

    let procs = PgProc::load_all(&mut conn).expect("Failed to load procs");
    for proc in procs.iter().take(10) {
        // Limit to 10 to avoid slow tests
        let _ = proc.argument_types(&mut conn);
        let _ = proc.return_type(&mut conn);
        let _ = proc.extension(&mut conn);
    }

    let extensions = PgExtension::load_all(&mut conn).expect("Failed to load extensions");
    for ext in extensions {
        let _ = ext.functions(&mut conn);
        let _ = ext.types(&mut conn);
        let _ = ext.enums(&mut conn);

        let _ = PgExtension::load(&ext.extname, &mut conn);
    }

    for table in db.tables() {
        for column in table.columns(&db) {
            // column is &Column. We need to call methods on it.
            // The methods take &self.
            let _ = column.metadata(Rc::new(table.clone()), &mut conn);
            let _ = column.check_constraints(&mut conn);
            let _ = column.geometry(&mut conn);
            let _ = column.geography(&mut conn);
            let _ = column.has_foreign_keys(&mut conn);

            if let Ok(pg_type) = column.pg_type(&mut conn) {
                let _ = pg_type.extension(&mut conn);
                let _ = pg_type.internal_user_defined_types(&mut conn);
                let _ = pg_type.base_type(&mut conn);
                let _ = pg_type.is_user_defined(&mut conn);
                if let Ok(attributes) = pg_type.attributes(&mut conn) {
                    for attr in attributes {
                        let _ = attr.pg_type(&mut conn);
                    }
                }
                let _ = pg_type.variants(&mut conn);

                let _ = pg_type.postgres_type(&mut conn);
            }
        }

        for check in <Table as TableLike>::check_constraints(table, &db) {
            let _ = CheckConstraintLike::expression(check, &db);
            let _ = CheckConstraintLike::table(check, &db);
            let _ = CheckConstraintLike::columns(check, &db);
        }

        for fk in <Table as TableLike>::foreign_keys(table, &db) {
            let _ = ForeignKeyLike::foreign_key_name(fk);
            let _ = ForeignKeyLike::referenced_table(fk, &db);
            let _ = ForeignKeyLike::host_table(fk, &db);
            let _ = ForeignKeyLike::referenced_columns(fk, &db);
            let _ = ForeignKeyLike::host_columns(fk, &db);
            let _ = ForeignKeyLike::on_delete_cascade(fk, &db);
            let _ = ForeignKeyLike::match_kind(fk, &db);
        }

        for idx in <Table as TableLike>::unique_indices(table, &db) {
            let _ = UniqueIndexLike::table(idx, &db);
            let _ = UniqueIndexLike::expression(idx, &db);
        }
    }

    docker.stop().await.unwrap();
}
