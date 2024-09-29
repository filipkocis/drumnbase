use std::{collections::HashMap, path::Path, sync::{Arc, RwLock}};

use crate::{utils::{log, disk}, database::{DatabaseBuilder, Database, Run}, basics::Value, auth::{Role, Privilege, User}};

use super::{super::{Cluster, ClusterSettings}, ClusterBuilder};

impl ClusterBuilder {
    pub fn load(&self) -> Result<Cluster, String> {
        log::info(format!("loading cluster '{}'", self.name));

        let path = self.path();
        let path = Path::new(&path).to_str().ok_or("invalid path")?;
        
        if !disk::exists(&path) {
            let err_msg = format!("cluster '{}' does not exist", self.name);
            log::error(&err_msg); 
            return Err(err_msg) 
        }

        let directories = disk::get_directories(path)?;
        let mut databases = HashMap::new();
        let mut internal = None;

        // load databases
        for name in directories {
            log::info(&format!("loading database '{}' into cluster", name));
            let db = DatabaseBuilder::new().name(&name).root_dir(path).load()?;
            let db = Arc::new(RwLock::new(db));
            databases.insert(name.clone(), db.clone()); 

            if name == Self::INTERNAL_DB_NAME {
                if let Some(_) = internal.replace(db) {
                    let err_msg = "multiple internal databases found".to_owned();
                    log::error(&err_msg);
                    return Err(err_msg)
                }
            } 
        }

        let internal = internal.ok_or("internal database not found")?;
        let settings = ClusterSettings::new(&self.name, &self.root_dir);

        let roles = Self::load_roles(internal.clone())?;
        let users = Self::load_users(internal.clone(), &roles)?;

        let cluster = Cluster {
            databases,
            internal,
            settings,
            roles,
            users,
        };

        log::success("cluster loaded");
        Ok(cluster)
    }

    fn load_roles(internal: Arc<RwLock<Database>>) -> Result<HashMap<String, Role>, String> {
        log::info("loading internal roles");

        let query_result = Database::run(internal, Cluster::root_user_rc(), "query roles select *".to_owned())
            .or(Err("failed to query roles".to_owned()))?; 
        let mut roles = HashMap::new();
        
        // parse roles
        match query_result.data {
            Value::Array(array) => {
                for row in array {
                    let row = row.as_array().ok_or("invalid role row")?;

                    // TODO: dont use indexes, but use column table index
                    // TODO: when variable docs or joins are implemented, use them here
                    // TODO: implement row.get("name")
                    let name = row[0].as_text().ok_or("invalid role name")?; 
                    let object = row[1].as_text().ok_or("invalid privilege object")?;
                    let object_name = row[2].as_text().ok_or("invalid privilege name")?;
                    let action = row[3].as_text().ok_or("invalid privilege action")?;
                    let extra = row[4].as_text().map(|x| x.as_str());

                    let role = roles.entry(name.to_owned()).or_insert(Role::new(name));
                    let privilege = Privilege::from_fields(object, object_name, action, extra)?;

                    role.add_privilege(privilege);
                }
            },
            _ => {
                let err_msg = "invalid roles query result".to_owned();
                log::error(&err_msg);
                return Err(err_msg)
            }
        }

        log::success("roles loaded");
        Ok(roles)
    }

    fn load_users(internal: Arc<RwLock<Database>>, roles: &HashMap<String, Role>) -> Result<HashMap<String, User>, String> {
        log::info("loading internal users");

        let query_result = Database::run(internal, Cluster::root_user_rc(), "query users select *".to_owned())
            .or(Err("failed to query users".to_owned()))?; 
        let mut users = HashMap::new();
        
        // parse roles
        match query_result.data {
            Value::Array(array) => {
                for row in array {
                    let row = row.as_array().ok_or("invalid role row")?;

                    // TODO: dont use indexes, but use column table index
                    // TODO: when variable docs or joins are implemented, use them here
                    // TODO: implement row.get("name")
                    let name = row[0].as_text().ok_or("invalid user name")?; 
                    let hash = row[1].as_text().ok_or("invalid user hash")?;
                    let role_name = row[2].as_text().ok_or("invalid role name");
                    let is_superuser = row[3].as_bool().ok_or("invalid user is_superuser")?;

                    let user = users.entry(name.to_owned()).or_insert(User::new(name, hash));

                    if is_superuser {
                        user.is_superuser = true;
                    }

                    if role_name.is_err() && row[2] == Value::Null { continue }
                    let role_name = role_name?;

                    if let Some(role) = roles.get(role_name) {
                        user.add_role(role.clone());
                    } else {
                        let err_msg = format!("role '{}' not found", role_name);
                        log::error(&err_msg);
                        return Err(err_msg)
                    }
                }
            },
            _ => {
                let err_msg = "invalid users query result".to_owned();
                log::error(&err_msg);
                return Err(err_msg)
            }
        }

        if !users.contains_key(Self::INTERNAL_SUPERUSER_NAME) {
            let err_msg = "internal superuser not found".to_owned();
            log::error(&err_msg);
            return Err(err_msg)
        }

        log::success("users loaded");
        Ok(users)
    }
}
