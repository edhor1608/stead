use stead_module_sdk::{ContextFragment, ContextGenerator, ContextProvider, ContextProviderError};

struct PrimaryProvider;

impl ContextProvider for PrimaryProvider {
    fn name(&self) -> &'static str {
        "primary"
    }

    fn generate(&self, _prompt: &str) -> Result<String, ContextProviderError> {
        Ok("ok".to_string())
    }
}

#[test]
fn includes_sorted_citations_with_confidence_for_primary_path() {
    let generator = ContextGenerator::new(Box::new(PrimaryProvider), None);

    let fragments = vec![
        ContextFragment::new("z", "later", "docs/z.md"),
        ContextFragment::new("a", "first", "docs/a.md"),
    ];

    let context = generator.generate("Task", &fragments);

    let citation_ids: Vec<&str> = context
        .citations
        .iter()
        .map(|citation| citation.source_id.as_str())
        .collect();

    assert_eq!(citation_ids, vec!["a", "z"]);
    assert_eq!(context.citations[0].citation, "docs/a.md");
    assert_eq!(context.confidence, 0.9);
}
