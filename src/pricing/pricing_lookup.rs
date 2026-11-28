use super::model_cost::ModelCost;

pub(super) trait PricingSource: Sync {
    fn lookup(&self, provider: &str, model: &str) -> Option<ModelCost>;
}
