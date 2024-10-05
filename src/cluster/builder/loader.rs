use std::{collections::HashMap, path::Path, sync::{Arc, RwLock}, rc::Rc};

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
            let db = DatabaseBuilder::new(&name, path).load()?;
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

        let roles = Self::load_roles(internal.clone(), &settings)?;
        let users = Self::load_users(internal.clone(), &roles, &settings)?;

        let roles = roles.into_iter().map(|(_, role)| (role.name.clone(), role)).collect();

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

    fn load_roles(internal: Arc<RwLock<Database>>, settings: &ClusterSettings) -> Result<HashMap<u64, Role>, String> {
        log::info("loading internal roles");
        let result = Self::run_query("query roles select *", "roles", internal.clone(), settings)?;

        // parse roles
        let mut roles = HashMap::new();
        for row in result {
            let row = row.as_array().ok_or("invalid role row")?;

            // TODO: dont use indexes, but use column table index
            // TODO: when variable docs or joins are implemented, use them here
            // TODO: implement row.get("name")
            let id = row[0].as_numeric().ok_or("invalid role id")?.to_i128() as u64;
            let name = row[1].as_text().ok_or("invalid role name")?; 
            // TODO: implement field description

            let role = roles.entry(id).or_insert(Role::new(name));

            let query = format!("query privileges select * where role_id == {}", id);
            let result = Self::run_query(&query, "privileges", internal.clone(), settings)?;

            for row in result {
                let row = row.as_array().ok_or("invalid privilege row")?;

                let object = row[2].as_text().ok_or("invalid privilege object")?;
                let object_name = row[3].as_text().ok_or("invalid privilege name")?;
                let action = row[4].as_text().ok_or("invalid privilege action")?;
                let extra = row[5].as_text().map(|x| x.as_str());

                let privilege = Privilege::from_fields(object, object_name, action, extra)?;
                role.add_privilege(privilege);
            }
        }

        log::success("roles loaded");
        Ok(roles)
    }

    fn load_users(internal: Arc<RwLock<Database>>, roles: &HashMap<u64, Role>, settings: &ClusterSettings) -> Result<HashMap<String, User>, String> {
        log::info("loading internal users");
        let result = Self::run_query("query users select *", "users", internal.clone(), settings)?;

        // parse users
        let mut users = HashMap::new();
        for row in result {
            let row = row.as_array().ok_or("invalid user row")?;

            // TODO: dont use indexes, but use column table index
            // TODO: when variable docs or joins are implemented, use them here
            // TODO: implement row.get("name")
            let id = row[0].as_numeric().ok_or("invalid user id")?.to_i128() as u64; 
            let name = row[1].as_text().ok_or("invalid user name")?; 
            let hash = row[2].as_text().ok_or("invalid user hash")?;
            let is_superuser = row[3].as_bool().ok_or("invalid user is_superuser")?;

            let user = users.entry(name.to_owned()).or_insert(User::new(name, hash));
            if is_superuser {
                user.is_superuser = true;
            }

            let query = format!("query user_roles select * where user_id == {}", id);
            let result = Self::run_query(&query, "user_roles", internal.clone(), settings)?;

            for row in result {
                let row = row.as_array().ok_or("invalid user_role row")?;

                let role_id = row[2].as_numeric().ok_or("invalid user_role role_id")?.to_i128() as u64;
                let role = roles.get(&role_id).ok_or("role from user_roles not found")?;

                user.add_role(role.clone());
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

    /// Helper function to run a query and extract data from it's result
    ///
    /// 'what' is a description of the item we are querying, used for error messages
    fn run_query(query: &str, what: &str, internal: Arc<RwLock<Database>>, settings: &ClusterSettings) -> Result<Vec<Value>, String> {
        let options = Cluster::root_run_options(internal.clone(), settings);
        let query_result = Database::run(internal, query.to_string(), Rc::new(options))
            .or(Err(format!("failed to query {}", what)))?; 

        let result;
        
        match query_result.data {
            Value::Array(array) => {
                for row in &array {
                    row.as_array().ok_or(&format!("invalid {} row", what))?;
                }

                result = array;
            },
            _ => {
                let msg = format!("invalid {} query result", what);
                log::error(&msg);
                return Err(msg)
            }
        }

        Ok(result)
    }
}
