use super::model_id::ParsedModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelList(Vec<ParsedModel>);

impl ModelList {
    pub fn new(models: Vec<ParsedModel>) -> Result<Self, String> {
        if models.is_empty() {
            return Err("model list must not be empty".into());
        }
        Ok(Self(models))
    }

    #[must_use]
    pub fn first(&self) -> &ParsedModel {
        &self.0[0]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ParsedModel> {
        self.0.iter()
    }

    #[must_use]
    pub fn cli_value(&self) -> String {
        self.iter()
            .map(ParsedModel::canonical)
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl<'a> IntoIterator for &'a ModelList {
    type Item = &'a ParsedModel;
    type IntoIter = std::slice::Iter<'a, ParsedModel>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<ParsedModel> for ModelList {
    fn from(model: ParsedModel) -> Self {
        Self(vec![model])
    }
}

pub fn split_model_list(raw: &str) -> Result<Vec<&str>, String> {
    let mut depth = 0_i32;
    let mut start = 0;
    let mut models = Vec::new();
    for (index, ch) in raw.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                models.push(raw[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    models.push(raw[start..].trim());
    if models.contains(&"") {
        return Err("model list must not contain empty entries".into());
    }
    Ok(models)
}

#[cfg(test)]
#[path = "model_list_tests.rs"]
mod tests;
