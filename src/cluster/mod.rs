mod builder;
mod settings;

pub use builder::ClusterBuilder;
pub use settings::ClusterSettings;

use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{database::Database, auth::{Role, User}};

pub struct Cluster {
    pub databases: HashMap<String, Arc<RwLock<Database>>>,
    pub roles: HashMap<String, Role>,
    pub users: HashMap<String, User>,

    pub internal: Arc<RwLock<Database>>,
    pub settings: ClusterSettings,
}

impl Default for Cluster {
    fn default() -> Self {
        Self {
            databases: HashMap::default(),
            roles: HashMap::default(),
            users: HashMap::default(),
            internal: Arc::new(RwLock::new(Database::default())),
            settings: ClusterSettings::default(),
        }
    }
}
