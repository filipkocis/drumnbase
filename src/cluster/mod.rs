mod builder;
mod settings;

pub use builder::ClusterBuilder;
pub use settings::ClusterSettings;

use std::{collections::HashMap, sync::{Arc, RwLock}, rc::Rc};

use crate::{database::{Database, RunOptions, Run, QueryResult, DatabaseBuilder}, auth::{Role, User, Hashish}, utils::is_valid_name, syntax::context::Ctx};

pub struct Cluster {
    pub databases: HashMap<String, Arc<RwLock<Database>>>,
    pub roles: HashMap<String, Role>,
    pub users: HashMap<String, User>,

    pub internal: Arc<RwLock<Database>>,
    pub settings: ClusterSettings,
}

impl Cluster {
    pub fn from_settings_unloaded(internal: Arc<RwLock<Database>>, settings: &ClusterSettings) -> Self {
        Self {
            databases: HashMap::default(),
            roles: HashMap::default(),
            users: HashMap::default(),
            internal: internal,
            settings: settings.clone(),
        }
    }

    pub fn root_user() -> User {
        let mut user = User::new("root", "");
        user.is_superuser = true;
        user
    }

    pub fn root_user_rc() -> Rc<User> {
        Rc::new(Self::root_user())
    }

    /// Return default RunOptions for root cluster user, 
    /// should be used for internal loading processes only
    pub fn root_run_options(internal: Arc<RwLock<Database>>, settings: &ClusterSettings) -> RunOptions {
        let cluster = Cluster::from_settings_unloaded(internal, settings);

        RunOptions {
            cluster_user: Self::root_user_rc(),
            auth_user: Self::root_user_rc(),
            cluster: Arc::new(RwLock::new(cluster)),
            is_schema: false,
        }
    }

    /// Run a query as root cluster user
    fn run_as_root(&self, query: String) -> Result<QueryResult, String> {
        let options = Cluster::root_run_options(self.internal.clone(), &self.settings);
        Database::run(self.internal.clone(), query, Rc::new(options))
    }

    /// Create a new physical database in the cluster
    pub fn create_database(&mut self, name: &str, ctx: &Ctx) -> Result<(), String> {
        if ctx.is_schema() {
            return Err("Can't create database in schema context".to_string())
        }

        if self.databases.contains_key(name) {
            return Err(format!("Database {} already exists", name))
        }

        if !is_valid_name(name) {
            return Err("Database name invalid".to_string())
        }

        let database = DatabaseBuilder::new(name, &self.settings.root_dir).create()?;
        self.databases.insert(name.to_string(), Arc::new(RwLock::new(database)));

        Ok(())
    }

    /// Create a new physical user in the cluster
    pub fn create_user(&mut self, name: &str, password: &str, is_superuser: bool, ctx: &Ctx) -> Result<(), String> {
        if self.users.contains_key(name) {
            return Err(format!("User {} already exists", name))
        }

        if !is_valid_name(name) {
            return Err("User name invalid".to_string())
        }

        let hash = Hashish::hash(password)?;
        let mut user = User::new(name, &hash);
        user.is_superuser = is_superuser;

        if !ctx.is_schema() { 
            let query = format!("query users insert name:'{}' hash:'{}' is_superuser:{}", user.name, user.hash, user.is_superuser);
            self.run_as_root(query)?;
        }
    
        self.users.insert(name.to_string(), user);
        Ok(())
    }

    /// Create a new physical role in the cluster
    pub fn create_role(&mut self, name: &str, ctx: &Ctx) -> Result<(), String> {
        if self.roles.contains_key(name) {
            return Err(format!("Role {} already exists", name))
        }

        if !is_valid_name(name) {
            return Err("Role name invalid".to_string())
        }

        let role = Role::new(name);

        if !ctx.is_schema() { 
            let query = format!("query roles insert name:'{}'", role.name);
            self.run_as_root(query)?;
        }

        self.roles.insert(name.to_string(), role);
        Ok(())
    }

    /// Grant a role to user
    pub fn grant_role(&mut self, role: &str, to: &str, ctx: &Ctx) -> Result<(), String> {
        if !self.roles.contains_key(role) {
            return Err(format!("Role {} does not exist", role))
        }

        if !self.users.contains_key(to) {
            return Err(format!("User {} does not exist", to))
        }

        if !ctx.is_schema() {
            // TODO: temporary update, change later
            let query = format!("query users update role_name:'{}' where name == {}", role, to);
            self.run_as_root(query)?;
        }

        let role = self.roles.get(role).unwrap().clone();
        self.users.get_mut(to).unwrap().add_role(role);

        Ok(())
    }
}
