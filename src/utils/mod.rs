pub mod disk;
pub mod log;
pub mod args;

pub fn is_valid_name(name: &str) -> bool {
    if name.is_empty() ||
        name.len() > 64 ||
        name.len() < 3 ||
        name.chars().next().unwrap().is_ascii_digit() {
        return false
    }

    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
