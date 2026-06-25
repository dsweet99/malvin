//! Run multiple CLI positional requests in sequence (each invocation is independent).

fn malvin_cmd_err(subcommand: &str, detail: &str) -> String {
    if subcommand.is_empty() {
        format!("malvin: {detail}")
    } else {
        format!("malvin {subcommand}: {detail}")
    }
}

pub(crate) fn run_sequential<F>(subcommand: &str, items: &[String], mut run_one: F) -> Result<(), String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    if items.is_empty() {
        return Err(malvin_cmd_err(
            subcommand,
            "missing required argument (text or path)",
        ));
    }
    for (index, item) in items.iter().enumerate() {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err(malvin_cmd_err(
                subcommand,
                &format!("empty argument at position {}", index + 1),
            ));
        }
        run_one(trimmed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_sequential;

    #[test]
    fn run_sequential_invokes_each_item_in_order() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut seen = Vec::new();
        run_sequential("test", &items, |s| {
            seen.push(s.to_string());
            Ok(())
        })
        .expect("ok");
        assert_eq!(seen, vec!["a", "b"]);
    }

    #[test]
    fn run_sequential_stops_on_first_error() {
        let items = vec!["ok".to_string(), "bad".to_string(), "never".to_string()];
        let mut count = 0;
        let err = run_sequential("test", &items, |s| {
            count += 1;
            if s == "bad" {
                Err("fail".into())
            } else {
                Ok(())
            }
        })
        .expect_err("should fail");
        assert_eq!(err, "fail");
        assert_eq!(count, 2);
    }

    #[test]
    fn run_sequential_rejects_empty_list() {
        let err = run_sequential("code", &[], |_| Ok(())).expect_err("empty");
        assert!(err.contains("code"));
        assert!(err.contains("missing"));
    }

    #[test]
    fn run_sequential_rejects_whitespace_only_item() {
        let items = vec!["good".to_string(), "   ".to_string()];
        let err = run_sequential("plan", &items, |_| Ok(())).expect_err("whitespace");
        assert!(err.contains("plan"));
        assert!(err.contains("empty argument"));
    }
}
