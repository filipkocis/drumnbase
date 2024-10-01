mod privilege;
mod role;
mod user;
mod hash;
mod authorize;
mod rls;

pub use privilege::*;
pub use role::Role;
pub use user::User;
pub use hash::Hashish;
pub use authorize::Authorize;
pub use rls::*;

use crate::cluster::Cluster;

use self::action::DatabaseAction;

impl Cluster {
    /// Authenticate a user with username and password on a database
    pub fn authenticate(&self, username: &str, password: &str, database: &str) -> Result<User, String> {
        let user = self.users.get(username).ok_or("user not found")?;
        let verified = Hashish::verify(password, &user.hash)?;
        
        let database = match self.databases.get(database) {
            Some(database) => database.clone(),
            None => return Err("database not found".to_string())
        };

        database.read().unwrap().authorize(user, DatabaseAction::Connect)?;

        if verified {
            Ok(user.clone())
        } else {
            Err("could not authenticate".to_owned())
        }
    }
}
