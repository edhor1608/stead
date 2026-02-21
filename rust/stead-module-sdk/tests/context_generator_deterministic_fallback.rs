use stead_module_sdk::{ContextFragment, ContextGenerator, ContextProvider, ContextProviderError};

struct UnavailableProvider;

impl ContextProvider for UnavailableProvider {
    fn name(&self) -> &'static str {
        "primary"
    }

    fn generate(&self, _prompt: &str) -> Result<String, ContextProviderError> {
        Err(ContextProviderError::Unavailable)
    }
}

#[test]
fn falls_back_deterministically_when_backend_is_unavailable() {
    let generator = ContextGenerator::new(Box::new(UnavailableProvider), None);

    let context = generator.generate("Task", &[ContextFragment::new("a", "context", "docs/a.md")]);

    assert_eq!(context.provider, "deterministic-fallback");
    assert_eq!(context.content, "fallback: deterministic context summary");
    assert!(context.used_fallback);
    assert_eq!(context.confidence, 0.4);
}
