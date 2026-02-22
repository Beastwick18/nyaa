use std::fmt::Display;

pub struct Category {
    label: String,
    values: Vec<String>,
}

impl Category {
    pub fn new(label: impl Display, values: impl IntoIterator<Item = impl Display>) -> Category {
        Category {
            label: label.to_string(),
            values: values.into_iter().map(|v| v.to_string()).collect(),
        }
    }
}
