use crate::utils::log;

mod loader;
mod creator;

pub struct ClusterBuilder {
    name: String,
    root_dir: String,
}

impl ClusterBuilder {
    pub fn new(name: &str, root_dir: &str) -> Self {
        if root_dir.is_empty() {
            log::error("missing field root_dir");
            panic!("missing field root_dir")
        }

        if name.is_empty() {
            log::error("missing field name");
            panic!("missing field name")
        }

        ClusterBuilder {
            name: name.to_owned(),
            root_dir: root_dir.to_owned(),
        }
    }

    pub fn path(&self) -> String {
        format!("{}/", self.root_dir) 
    }

    pub const INTERNAL_DB_NAME: &'static str = "drumnbase";
    pub const INTERNAL_SUPERUSER_NAME: &'static str = "drumnbase";
    pub const INTERNAL_DB_SCHEMA: &'static str = r#"
        create table users {
            id: u64, unique, required, default(seq("users", "id"));
            name: fixed(64), unique, required;
            hash: fixed(200), unique, required;
            is_superuser: bool, required, default(false);
            created_at: time(ms), required, default(now());
        };

        create table roles {
            id: u64, unique, required, default(seq("roles", "id"));
            name: fixed(64), unique, required;
            description: fixed(200);
            created_at: time(ms), required, default(now());
        };

        create table user_roles {
            id: u64, unique, required, default(seq("user_roles", "id"));
            user_id: u64, required;
            role_id: u64, required;
            created_at: time(ms), required, default(now());
        };

        create table privileges {
            id: u64, unique, required, default(seq("privileges", "id"));
            role_id: u64, required;
            object: fixed(64), required;
            object_name: fixed(64);
            action: fixed(64), required;
            extra: fixed(64);
        };
    "#;
}
