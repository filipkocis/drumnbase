mod builder;
mod settings;

pub use builder::ClusterBuilder;
pub use settings::ClusterSettings;

use std::{collections::HashMap, sync::{Arc, RwLock}, rc::Rc};

use crate::{database::Database, auth::{Role, User}};

pub struct Cluster {
    pub databases: HashMap<String, Arc<RwLock<Database>>>,
    pub roles: HashMap<String, Role>,
    pub users: HashMap<String, User>,

    pub internal: Arc<RwLock<Database>>,
    pub settings: ClusterSettings,
}

impl Cluster {
    pub fn root_user() -> User {
        let mut user = User::new("root", "");
        user.is_superuser = true;
        user
    }

    pub fn root_user_rc() -> Rc<User> {
        Rc::new(Self::root_user())
    }
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
