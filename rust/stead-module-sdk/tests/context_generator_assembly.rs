use stead_module_sdk::{ContextFragment, ContextGenerator, ContextProvider, ContextProviderError};

struct EchoProvider;

impl ContextProvider for EchoProvider {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn generate(&self, prompt: &str) -> Result<String, ContextProviderError> {
        Ok(format!("generated:{prompt}"))
    }
}

#[test]
fn assembles_prompt_deterministically_from_sorted_sources() {
    let generator = ContextGenerator::new(Box::new(EchoProvider), None);

    let fragments = vec![
        ContextFragment::new("b-doc", "Second fragment", "docs/b.md"),
        ContextFragment::new("a-doc", "First fragment", "docs/a.md"),
    ];

    let prompt = generator.assemble_prompt("Fix auth", &fragments);

    assert_eq!(
        prompt,
        "Task: Fix auth\n[a-doc] First fragment\n[b-doc] Second fragment"
    );
}
