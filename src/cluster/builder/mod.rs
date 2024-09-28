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
}
