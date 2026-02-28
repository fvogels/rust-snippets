pub struct SearchParameters {
    pub keywords: Vec<String>,
    pub tags: Vec<String>,
}

impl SearchParameters {
    pub fn new() -> Self {
        SearchParameters {
            keywords: Vec::new(),
            tags: Vec::new(),
        }
    }
}
