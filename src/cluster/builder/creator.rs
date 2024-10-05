use std::{path::Path, sync::{Arc, RwLock}, collections::HashMap, rc::Rc};

use crate::{utils::{log, disk}, cluster::{Cluster, ClusterSettings}, database::{DatabaseBuilder, Database, Run}, auth::{User, Hashish}};

use super::ClusterBuilder;

impl ClusterBuilder {
    /// Create a new cluster at self.path()
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

        let settings = ClusterSettings::new(&self.name, &path);
        let internal = Self::create_internal_database(&path, &settings)?;
        let users = Self::add_default_users(internal.clone(), password, &settings)?;

        let mut databases = HashMap::new();
        databases.insert(Self::INTERNAL_DB_NAME.to_owned(), internal.clone());
        
        let cluster = Cluster {
            databases,
            internal,
            settings,
            roles: Default::default(),
            users,
        };

        log::success("cluster created");
        Ok(cluster)
    }

    /// Create internal database for the cluster
    fn create_internal_database(root_dir: &str, settings: &ClusterSettings) -> Result<Arc<RwLock<Database>>, String> {
        log::info("creating internal database");

        let internal_name = Self::INTERNAL_DB_NAME;
        let internal_schema = Self::INTERNAL_DB_SCHEMA.to_owned();

        let internal = DatabaseBuilder::new(internal_name, root_dir).create()?;
        let internal = Arc::new(RwLock::new(internal));
        let options = Cluster::root_run_options(internal.clone(), settings);
        Database::run(internal.clone(), internal_schema, Rc::new(options))?;
        
        log::success("internal database created");
        Ok(internal)
    }

    /// Add default cluster users to the internal database
    fn add_default_users(internal: Arc<RwLock<Database>>, superuser_password: &str, settings: &ClusterSettings) -> Result<HashMap<String, User>, String> {
        log::info("adding default users");

        let mut users = HashMap::new();

        let hash = Hashish::hash(superuser_password)?;
        let mut user = User::new(Self::INTERNAL_SUPERUSER_NAME, &hash);
        user.is_superuser = true;

        let query = format!("query users insert name:'{}' hash:'{}' is_superuser:{}", user.name, user.hash, user.is_superuser);
        let options = Cluster::root_run_options(internal.clone(), settings);
        Database::run(internal, query, Rc::new(options))?;

        users.insert(user.name.clone(), user);

        log::success("default users added");
        Ok(users)
    }
}
