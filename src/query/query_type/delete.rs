use crate::query::condition::chain::ConditionChain;

#[derive(Debug)]
pub struct DeleteQuery {
    pub condition_chain: ConditionChain,
    pub limit: Option<usize>,
}

impl DeleteQuery {
    pub fn is_valid(&self) -> bool {
        !self.condition_chain.is_empty()
    }
}
