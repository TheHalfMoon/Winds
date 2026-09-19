use super::*;

fn owner(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn client(label: &str) -> ClientConnectionId {
    ClientConnectionId::new(label).unwrap()
}

#[test]
fn t154_simultaneous_requests_serialize_to_exactly_one_controller() {
    let runtime_id = runtime(1);
    let first = client("controller-a");
    let second = client("controller-b");
    let mut registry = ControllerRegistry::new(owner(1));
    registry.register_runtime(runtime_id);

    let granted = registry
        .request_control(runtime_id, first.clone(), 100)
        .unwrap()
        .expect("first explicit request must grant the empty lease");
    assert_eq!(granted.disposition, ControllerDisposition::Granted);
    assert_eq!(granted.identity.controller_client_id, first);

    let conflict = registry
        .request_control(runtime_id, second.clone(), 100)
        .unwrap_err();
    assert_eq!(
        conflict,
        ControllerError::ControllerConflict {
            active_controller: client("controller-a")
        }
    );
    assert_eq!(
        registry
            .authorize_mutation(runtime_id, &client("controller-a"), 101)
            .unwrap()
            .controller_client_id,
        client("controller-a")
    );
    assert_eq!(
        registry
            .authorize_mutation(runtime_id, &second, 101)
            .unwrap_err(),
        ControllerError::NotActiveController
    );
}

#[test]
fn t154_heartbeat_target_and_expiry_are_exact_and_renewal_extends_only_exact_lease() {
    let runtime_id = runtime(2);
    let controller = client("heartbeat-controller");
    let mut registry = ControllerRegistry::new(owner(2));
    registry.register_runtime(runtime_id);
    registry
        .request_control(runtime_id, controller.clone(), 1_000)
        .unwrap();

    let initial = registry.active_lease(runtime_id, 1_000).unwrap().unwrap();
    assert!(!initial.heartbeat_due(10_999).unwrap());
    assert!(initial.heartbeat_due(11_000).unwrap());
    assert_eq!(initial.expires_at_monotonic_ms, 31_000);

    let renewed = registry
        .renew_control(runtime_id, &controller, 30_999)
        .unwrap();
    assert_eq!(renewed.last_heartbeat_monotonic_ms, 30_999);
    assert_eq!(renewed.expires_at_monotonic_ms, 60_999);
    assert!(registry.expire_leases(31_000).unwrap().is_empty());

    let expired = registry.expire_leases(60_999).unwrap();
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].disposition, ControllerDisposition::Expired);
    assert_eq!(expired[0].identity.controller_client_id, controller);
    assert_eq!(
        registry
            .authorize_mutation(runtime_id, &client("heartbeat-controller"), 61_000)
            .unwrap_err(),
        ControllerError::NotActiveController
    );
}

#[test]
fn t154_disconnect_revokes_without_promoting_an_observer_and_old_client_cannot_renew() {
    let runtime_id = runtime(3);
    let controller = client("disconnect-controller");
    let observer = client("observer");
    let mut registry = ControllerRegistry::new(owner(3));
    registry.register_runtime(runtime_id);
    registry
        .request_control(runtime_id, controller.clone(), 0)
        .unwrap();

    let transitions = registry.disconnect_client(&controller, 5_000);
    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0].disposition,
        ControllerDisposition::Disconnected
    );
    assert_eq!(
        registry
            .renew_control(runtime_id, &controller, 5_001)
            .unwrap_err(),
        ControllerError::NotActiveController
    );

    let observer_state = registry.control_state(runtime_id, &observer, 5_001).unwrap();
    assert_eq!(observer_state.authority, ClientAuthority::Observer);
    assert!(observer_state.controller_client_id.is_none());
    assert!(observer_state.lease.is_none());

    let granted = registry
        .request_control(runtime_id, observer.clone(), 5_002)
        .unwrap()
        .expect("a new explicit request is required after disconnect");
    assert_eq!(granted.identity.controller_client_id, observer);
}

#[test]
fn t154_release_is_active_controller_only_and_never_auto_promotes_waiters() {
    let runtime_id = runtime(4);
    let controller = client("release-controller");
    let observer = client("release-observer");
    let mut registry = ControllerRegistry::new(owner(4));
    registry.register_runtime(runtime_id);
    registry
        .request_control(runtime_id, controller.clone(), 10)
        .unwrap();

    assert_eq!(
        registry
            .release_control(runtime_id, &observer, 11)
            .unwrap_err(),
        ControllerError::NotActiveController
    );
    let released = registry
        .release_control(runtime_id, &controller, 12)
        .unwrap();
    assert_eq!(released.disposition, ControllerDisposition::Released);
    assert_eq!(released.identity.controller_client_id, controller);

    let state = registry.control_state(runtime_id, &observer, 13).unwrap();
    assert_eq!(state.authority, ClientAuthority::Observer);
    assert!(state.controller_client_id.is_none());
    assert!(state.lease.is_none());
}

#[test]
fn t154_takeover_after_expiry_invalidates_every_old_controller_mutation_class() {
    let runtime_id = runtime(5);
    let old = client("old-controller");
    let new = client("new-controller");
    let mut registry = ControllerRegistry::new(owner(5));
    registry.register_runtime(runtime_id);
    registry.request_control(runtime_id, old.clone(), 0).unwrap();

    let expired = registry.expire_leases(CONTROLLER_LEASE_EXPIRY_MS).unwrap();
    assert_eq!(expired.len(), 1);
    let granted = registry
        .request_control(runtime_id, new.clone(), CONTROLLER_LEASE_EXPIRY_MS)
        .unwrap()
        .expect("first serialized request after expiry must win");
    assert_eq!(granted.identity.controller_client_id, new);

    for operation in ["INPUT", "RESIZE", "INTERRUPT", "STOP"] {
        assert_eq!(
            registry
                .authorize_mutation(
                    runtime_id,
                    &old,
                    CONTROLLER_LEASE_EXPIRY_MS.saturating_add(1)
                )
                .unwrap_err(),
            ControllerError::NotActiveController,
            "{operation} must reject the prior controller after takeover"
        );
        assert!(
            registry
                .authorize_mutation(
                    runtime_id,
                    &client("new-controller"),
                    CONTROLLER_LEASE_EXPIRY_MS.saturating_add(1)
                )
                .is_ok(),
            "{operation} must accept only the new exact lease"
        );
    }
}

#[test]
fn t154_expiry_and_disconnect_on_one_runtime_leave_peer_runtime_controller_unchanged() {
    let first_runtime = runtime(6);
    let peer_runtime = runtime(7);
    let first_controller = client("first-controller");
    let peer_controller = client("peer-controller");
    let mut registry = ControllerRegistry::new(owner(6));
    registry.register_runtime(first_runtime);
    registry.register_runtime(peer_runtime);
    registry
        .request_control(first_runtime, first_controller.clone(), 0)
        .unwrap();
    registry
        .request_control(peer_runtime, peer_controller.clone(), 15_000)
        .unwrap();

    let expired = registry.expire_leases(30_000).unwrap();
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].identity.runtime_namespace_id, first_runtime);
    assert!(
        registry
            .authorize_mutation(peer_runtime, &peer_controller, 30_000)
            .is_ok()
    );

    assert!(registry.disconnect_client(&first_controller, 30_001).is_empty());
    assert!(
        registry
            .authorize_mutation(peer_runtime, &peer_controller, 30_001)
            .is_ok()
    );
}

#[test]
fn t154_owner_revocation_is_attributable_and_generation_restart_never_inherits_authority() {
    let runtime_id = runtime(8);
    let controller = client("restart-controller");
    let first_generation = owner(8);
    let replacement_generation = owner(9);

    let mut first_registry = ControllerRegistry::new(first_generation);
    first_registry.register_runtime(runtime_id);
    first_registry
        .request_control(runtime_id, controller.clone(), 100)
        .unwrap();
    let revoked = first_registry
        .owner_revoke(runtime_id, 200)
        .unwrap()
        .expect("owner-defined revocation must identify the prior lease");
    assert_eq!(revoked.disposition, ControllerDisposition::OwnerRevoked);
    assert_eq!(revoked.identity.owner_generation_id, first_generation);
    assert_eq!(revoked.identity.controller_client_id, controller);

    let mut replacement_registry = ControllerRegistry::new(replacement_generation);
    replacement_registry.register_runtime(runtime_id);
    assert!(
        replacement_registry
            .active_lease(runtime_id, 201)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        replacement_registry
            .authorize_mutation(runtime_id, &client("restart-controller"), 201)
            .unwrap_err(),
        ControllerError::NotActiveController
    );

    let new_grant = replacement_registry
        .request_control(runtime_id, client("restart-controller"), 202)
        .unwrap()
        .unwrap();
    assert_eq!(
        new_grant.identity.owner_generation_id,
        replacement_generation
    );
    assert_ne!(new_grant.identity.owner_generation_id, first_generation);
}

#[test]
fn t154_monotonic_regression_fails_closed_without_extending_lease() {
    let runtime_id = runtime(9);
    let controller = client("clock-controller");
    let mut registry = ControllerRegistry::new(owner(10));
    registry.register_runtime(runtime_id);
    registry
        .request_control(runtime_id, controller.clone(), 10_000)
        .unwrap();
    assert_eq!(
        registry
            .renew_control(runtime_id, &controller, 9_999)
            .unwrap_err(),
        ControllerError::MonotonicTimeRegression
    );
    let lease = registry.active_lease(runtime_id, 10_000).unwrap().unwrap();
    assert_eq!(lease.last_heartbeat_monotonic_ms, 10_000);
    assert_eq!(lease.expires_at_monotonic_ms, 40_000);
}
