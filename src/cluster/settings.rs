use crate::utils::log;

#[derive(Debug)]
pub struct ClusterSettings {
    pub name: String,
    pub root_dir: String,
}

impl Default for ClusterSettings {
    fn default() -> Self {
        Self {
            name: String::from(""),
            root_dir: String::from("./data"),
        }
    }
}

impl ClusterSettings {
    pub fn new(name: &str, root_dir: &str) -> Self {
        if root_dir.is_empty() {
            log::error("missing field root_dir");
            panic!("missing field root_dir")
        }

        if name.is_empty() {
            log::error("missing field name");
            panic!("missing field name")
        }

        Self {
            name: name.to_owned(),
            root_dir: root_dir.to_owned(),
        }
    }
}
