use stead_module_sdk::{ModuleError, ModuleManager, ModuleName};

#[test]
fn session_proxy_can_be_disabled_and_reenabled() {
    let mut modules = ModuleManager::default();

    assert!(modules.is_enabled(ModuleName::SessionProxy));

    modules.disable(ModuleName::SessionProxy);
    assert!(!modules.is_enabled(ModuleName::SessionProxy));
    assert!(matches!(
        modules.ensure_enabled(ModuleName::SessionProxy),
        Err(ModuleError::ModuleDisabled(ModuleName::SessionProxy))
    ));

    modules.enable(ModuleName::SessionProxy);
    assert!(modules.ensure_enabled(ModuleName::SessionProxy).is_ok());
}

#[test]
fn disabling_optional_module_does_not_block_core_operations() {
    let mut modules = ModuleManager::default();
    modules.disable(ModuleName::SessionProxy);

    let value = modules.run_core_operation(|| 7 * 6);
    assert_eq!(value, 42);
}
