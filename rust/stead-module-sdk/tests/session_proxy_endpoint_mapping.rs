use stead_module_sdk::{project_endpoint_name, ModuleManager, ModuleName, SessionProxy};

#[test]
fn enabled_session_proxy_maps_project_to_deterministic_endpoint() {
    let mut proxy = SessionProxy::default();
    let modules = ModuleManager::default();

    let first = proxy
        .resolve_project_endpoint(&modules, "/workspace/project-alpha", "agent-a")
        .unwrap()
        .expect("session proxy enabled by default");

    let second = proxy
        .resolve_project_endpoint(&modules, "/workspace/project-alpha", "agent-a")
        .unwrap()
        .expect("mapping should remain available");

    assert_eq!(first.name, second.name);
    assert_eq!(first.port, second.port);
    assert_eq!(first.url, second.url);
    assert!(first.url.contains(".localhost:"));
}

#[test]
fn disabled_session_proxy_returns_none_without_core_regressions() {
    let mut proxy = SessionProxy::default();
    let mut modules = ModuleManager::default();
    modules.disable(ModuleName::SessionProxy);

    let endpoint = proxy
        .resolve_project_endpoint(&modules, "/workspace/project-alpha", "agent-a")
        .unwrap();

    assert!(endpoint.is_none());
    assert_eq!(modules.run_core_operation(|| 42), 42);
}

#[test]
fn endpoint_mapping_is_project_scoped() {
    let mut proxy = SessionProxy::default();
    let modules = ModuleManager::default();

    let alpha = proxy
        .resolve_project_endpoint(&modules, "/workspace/project-alpha", "agent-a")
        .unwrap()
        .expect("alpha mapping should exist");

    let beta = proxy
        .resolve_project_endpoint(&modules, "/workspace/project-beta", "agent-a")
        .unwrap()
        .expect("beta mapping should exist");

    assert_ne!(alpha.name, beta.name);
    assert_ne!(alpha.url, beta.url);
}

#[test]
fn project_endpoint_name_matches_session_proxy_output() {
    let mut proxy = SessionProxy::default();
    let modules = ModuleManager::default();
    let project = "/workspace/project-alpha";

    let endpoint = proxy
        .resolve_project_endpoint(&modules, project, "agent-a")
        .unwrap()
        .expect("endpoint mapping should exist");

    assert_eq!(endpoint.name, project_endpoint_name(project));
}
