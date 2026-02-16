use stead_daemon::{ApiRequest, ApiResponse, Daemon, API_VERSION};

#[test]
fn endpoint_api_claim_list_release_flow_is_versioned() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::with_port_range(&db, 4100, 4102).unwrap();

    let claim = daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "api".to_string(),
            owner: "agent-a".to_string(),
            port: Some(4100),
        })
        .unwrap();

    assert_eq!(claim.version, API_VERSION);
    match claim.data {
        ApiResponse::EndpointClaim(result) => {
            assert!(format!("{result:?}").contains("api"));
        }
        other => panic!("unexpected claim response: {other:?}"),
    }

    let listed = daemon.handle(ApiRequest::ListEndpoints).unwrap();
    assert_eq!(listed.version, API_VERSION);

    match listed.data {
        ApiResponse::Endpoints(leases) => {
            assert_eq!(leases.len(), 1);
            assert_eq!(leases[0].name, "api");
            assert_eq!(leases[0].owner, "agent-a");
            assert_eq!(leases[0].port, 4100);
        }
        other => panic!("unexpected list response: {other:?}"),
    }

    let released = daemon
        .handle(ApiRequest::ReleaseEndpoint {
            name: "api".to_string(),
            owner: "agent-a".to_string(),
        })
        .unwrap();

    match released.data {
        ApiResponse::EndpointReleased(lease) => {
            assert_eq!(lease.name, "api");
            assert_eq!(lease.owner, "agent-a");
        }
        other => panic!("unexpected release response: {other:?}"),
    }
}

#[test]
fn endpoint_api_returns_typed_errors_for_not_owner_not_found_and_exhausted() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::with_port_range(&db, 4100, 4100).unwrap();

    daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "api".to_string(),
            owner: "agent-a".to_string(),
            port: Some(4100),
        })
        .unwrap();

    let not_owner = daemon
        .handle(ApiRequest::ReleaseEndpoint {
            name: "api".to_string(),
            owner: "agent-b".to_string(),
        })
        .expect_err("release by non-owner should fail");
    assert_eq!(not_owner.code, "not_owner");

    let not_found = daemon
        .handle(ApiRequest::ReleaseEndpoint {
            name: "missing".to_string(),
            owner: "agent-a".to_string(),
        })
        .expect_err("release of missing endpoint should fail");
    assert_eq!(not_found.code, "not_found");

    let exhausted = daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "web".to_string(),
            owner: "agent-c".to_string(),
            port: Some(4100),
        })
        .expect_err("exhausted range should return typed error");

    assert_eq!(exhausted.code, "endpoint_range_exhausted");
}
