mod builder;
mod settings;

pub use builder::ClusterBuilder;
pub use settings::ClusterSettings;

use std::{collections::HashMap, sync::{Arc, RwLock}, rc::Rc};

use crate::{database::{Database, RunOptions}, auth::{Role, User}};

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

    pub fn root_run_options(internal: Arc<RwLock<Database>>, settings: &ClusterSettings) -> RunOptions {
        let cluster = Cluster::from_settings_unloaded(internal, settings);

        RunOptions {
            cluster_user: Self::root_user_rc(),
            auth_user: Self::root_user_rc(),
            cluster: Arc::new(RwLock::new(cluster))
        }
    }
}
