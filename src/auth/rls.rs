use crate::{syntax::ast::Node, basics::Table};

use super::User;

#[derive(Debug)]
pub struct RlsPolicy {
    pub name: String,
    pub action: RlsAction,
    pub condition: Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RlsAction {
    Select,
    Insert,
    Update,
    Delete,
    All,
}

impl RlsPolicy {
    pub fn new(name: &str, action: RlsAction, condition: Node) -> Self {
        Self {
            name: name.to_owned(),
            action,
            condition,
        }
    }
}

impl Table {
    pub fn police(&self, user: &User, action: RlsAction) -> Vec<&Node> {
        if !self.rls_enabled || user.is_superuser {
            return vec![];
        }

        self.policies
            .iter()
            .filter_map(|(_, policy)| {
                if policy.action == action || policy.action == RlsAction::All || action == RlsAction::All {
                    Some(&policy.condition)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
