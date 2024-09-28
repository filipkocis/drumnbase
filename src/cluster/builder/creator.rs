use std::{path::Path, sync::{Arc, RwLock}, collections::HashMap};

use crate::{utils::{log, disk}, cluster::{Cluster, ClusterSettings}, database::{DatabaseBuilder, Database, Run}, auth::{User, Hashish}, random::Random};

use super::ClusterBuilder;

impl ClusterBuilder {
    pub fn create(&self, password: &str) -> Result<Cluster, String> {
        log::info(format!("creating cluster '{}'", self.name));

        let path = self.path();
        let path = Path::new(&path).to_str().ok_or("invalid path")?;

        if disk::exists(&path) {
            let err_msg = format!("cluster '{}' at '{}' already exists", self.name, path);
            log::error(&err_msg);
            return Err(err_msg)
        }

        disk::create_directory_all(&path)?;

        let internal = Self::create_internal_database(&path)?;
        let internal = Arc::new(RwLock::new(internal)); 
        let settings = ClusterSettings::new(&self.name, &path);
        let users = Self::add_default_users(internal.clone(), password)?;
        
        let cluster = Cluster {
            databases: Default::default(),
            internal,
            settings,
            roles: Default::default(),
            users,
        };

        log::success("cluster created");
        Ok(cluster)
    }

    fn create_internal_database(root_dir: &str) -> Result<Database, String> {
        log::info("creating internal database");

        let internal_name = Self::INTERNAL_DB_NAME;
        let schema_path = format!("{}/.temp-internal-schema", root_dir);

        disk::write_file(&schema_path, INTERNAL_DB_SCHEMA)?;
        
        let internal = DatabaseBuilder::new()
            .name(internal_name)
            .root_dir(root_dir)
            .create(&schema_path)?;
        
        disk::remove_file(&schema_path)?;
        
        log::success("internal database created");
        Ok(internal)
    }

    fn add_default_users(internal: Arc<RwLock<Database>>, superuser_password: &str) -> Result<HashMap<String, User>, String> {
        log::info("adding default users");

        let mut users = HashMap::new();

        let hash = Hashish::hash(superuser_password)?;
        let mut user = User::new(Self::INTERNAL_SUPERUSER_NAME, &hash);
        user.is_superuser = true;

        let query = format!("query users insert name:'{}' hash:'{}' is_superuser:{}", user.name, user.hash, user.is_superuser);
        Database::run(internal, query)?;

        users.insert(user.name.clone(), user);

        log::success("default users added");
        Ok(users)
    }
}

const INTERNAL_DB_SCHEMA: &str = r#"
table users create
table users column name add fixed(64) not_null unique
table users column hash add fixed(200) not_null unique
table users column role_name add fixed(64)
table users column is_superuser add bool default=false

table roles create
table roles column name add fixed(64) not_null
table roles column object add fixed(64) not_null
table roles column object_name add fixed(64) not_null
table roles column action add fixed(64) not_null
table roles column extra add fixed(64)
"#;
