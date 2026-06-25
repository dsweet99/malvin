use serde_json::Value;

pub const RUN_COST_SUMMARY_PREFIX: &str = "COST: ";

#[must_use]
pub fn format_cost_stdout_line_from_json(json: &Value) -> Option<String> {
    let cost = json.get("cost")?;
    let total = cost.get("total_cost")?.as_f64()?;
    let mean = cost.get("mean_cost_per_tx")?.as_f64()?;
    let median = cost.get("median_cost_per_tx")?.as_f64()?;
    let max = cost.get("max_cost_per_tx")?.as_f64()?;
    Some(format!(
        "{RUN_COST_SUMMARY_PREFIX}total_cost = {total:.4} mean_cost_per_tx = {mean:.4} median_cost_per_tx = {median:.4} max_cost_per_tx = {max:.4}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cost_stdout_line_formats_four_decimal_places() {
        let json = json!({
            "cost": {
                "total_cost": 0.0842,
                "mean_cost_per_tx": 0.0042,
                "median_cost_per_tx": 0.0031,
                "max_cost_per_tx": 0.0190
            }
        });
        let line = format_cost_stdout_line_from_json(&json).expect("line");
        assert!(line.contains("total_cost = 0.0842"));
        assert!(line.contains("mean_cost_per_tx = 0.0042"));
    }
}
