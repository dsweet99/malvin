use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct FactsArgs {
    #[command(subcommand)]
    pub command: FactsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum FactsCommand {
    /// Append a fact; fields are `field_name=value` in any order
    #[command(override_usage = "malvin admin facts add FIELD=VALUE...")]
    Add(AddFactArgs),
    /// Remove the fact with the given id
    #[command(override_usage = "malvin admin facts remove FACT_ID")]
    Remove(RemoveFactArgs),
}

#[derive(Args, Debug, Clone)]
pub struct RemoveFactArgs {
    /// The `fact_id` of the fact to remove
    #[arg(value_name = "FACT_ID")]
    pub fact_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct AddFactArgs {
    /// Fields as `field_name=value` (`fact_id`, `fact`, `symptoms`; symptoms may repeat)
    #[arg(value_name = "FIELD=VALUE", required = true, num_args = 1..)]
    pub fields: Vec<String>,
}
