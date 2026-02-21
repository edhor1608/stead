use stead_module_sdk::{ContextFragment, ContextGenerator, ContextProvider, ContextProviderError};

struct AlwaysAvailableProvider;

impl ContextProvider for AlwaysAvailableProvider {
    fn name(&self) -> &'static str {
        "primary"
    }

    fn generate(&self, _prompt: &str) -> Result<String, ContextProviderError> {
        Ok("primary output".to_string())
    }
}

struct UnavailableProvider;

impl ContextProvider for UnavailableProvider {
    fn name(&self) -> &'static str {
        "unavailable"
    }

    fn generate(&self, _prompt: &str) -> Result<String, ContextProviderError> {
        Err(ContextProviderError::Unavailable)
    }
}

struct FallbackProvider;

impl ContextProvider for FallbackProvider {
    fn name(&self) -> &'static str {
        "openrouter-fallback"
    }

    fn generate(&self, _prompt: &str) -> Result<String, ContextProviderError> {
        Ok("fallback output".to_string())
    }
}

#[test]
fn uses_primary_provider_when_available() {
    let generator = ContextGenerator::new(Box::new(AlwaysAvailableProvider), None);
    let context = generator.generate("Task", &[ContextFragment::new("a", "ctx", "doc")]);

    assert_eq!(context.provider, "primary");
    assert_eq!(context.content, "primary output");
    assert!(!context.used_fallback);
}

#[test]
fn uses_fallback_provider_when_primary_unavailable() {
    let generator = ContextGenerator::new(
        Box::new(UnavailableProvider),
        Some(Box::new(FallbackProvider)),
    );

    let context = generator.generate("Task", &[ContextFragment::new("a", "ctx", "doc")]);

    assert_eq!(context.provider, "openrouter-fallback");
    assert_eq!(context.content, "fallback output");
    assert!(context.used_fallback);
}
