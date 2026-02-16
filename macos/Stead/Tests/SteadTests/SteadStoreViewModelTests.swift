import XCTest
@testable import Stead

final class SteadStoreViewModelTests: XCTestCase {
    @MainActor
    func test_contract_upsert_adds_and_updates_single_row() {
        let store = SteadStore()
        let ready = makeContract(id: "c-1", status: .ready)
        let failed = makeContract(id: "c-1", status: .failed)

        store.apply(.contractUpsert(ready))
        XCTAssertEqual(store.contracts.count, 1)
        XCTAssertEqual(store.contracts.first?.status, .ready)

        store.apply(.contractUpsert(failed))
        XCTAssertEqual(store.contracts.count, 1)
        XCTAssertEqual(store.contracts.first?.status, .failed)
    }

    @MainActor
    func test_contract_remove_deletes_existing_row() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-1", status: .ready)))
        store.apply(.contractUpsert(makeContract(id: "c-2", status: .failed)))

        store.apply(.contractRemoved(id: "c-1"))

        XCTAssertEqual(store.contracts.map(\.id), ["c-2"])
    }

    @MainActor
    func test_event_driven_contracts_by_priority_keeps_attention_order() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-ready", status: .ready)))
        store.apply(.contractUpsert(makeContract(id: "c-failed", status: .failed)))
        store.apply(.contractUpsert(makeContract(id: "c-verifying", status: .verifying)))

        let labels = store.contractsByPriority.map(\.0)
        XCTAssertEqual(labels, ["Failed", "Verifying", "Ready"])
    }

    @MainActor
    func test_session_upsert_and_remove_are_event_driven() {
        let store = SteadStore()
        let session = SessionItem(
            id: "s-1",
            cli: .claude,
            projectPath: "/tmp/project",
            title: "Title",
            created: "2026-02-16T00:00:00Z",
            lastModified: "2026-02-16T00:01:00Z",
            messageCount: 3,
            gitBranch: "main"
        )

        store.apply(.sessionUpsert(session))
        XCTAssertEqual(store.sessions.map(\.id), ["s-1"])

        store.apply(.sessionRemoved(id: "s-1"))
        XCTAssertTrue(store.sessions.isEmpty)
    }

    @MainActor
    func test_menu_bar_state_idle_when_no_active_contracts() {
        let store = SteadStore()
        XCTAssertEqual(store.menuBarState, .idle)
    }

    @MainActor
    func test_menu_bar_state_running_when_executing_present() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-1", status: .executing)))
        XCTAssertEqual(store.menuBarState, .running)
    }

    @MainActor
    func test_menu_bar_state_attention_when_failed_present() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-1", status: .failed)))
        XCTAssertEqual(store.menuBarState, .attention)
    }

    @MainActor
    func test_menu_bar_state_decision_has_highest_priority() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-1", status: .failed)))
        store.apply(.contractUpsert(makeContract(id: "c-2", status: .verifying)))
        XCTAssertEqual(store.menuBarState, .decision)
    }

    @MainActor
    func test_primary_resolution_action_prefers_decision_contract() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-attn", status: .failed)))
        store.apply(.contractUpsert(makeContract(id: "c-decision", status: .verifying)))

        XCTAssertEqual(
            store.primaryResolutionAction,
            .resolveDecision(contractId: "c-decision")
        )
    }

    @MainActor
    func test_primary_resolution_action_returns_none_without_actionable_contract() {
        let store = SteadStore()
        store.apply(.contractUpsert(makeContract(id: "c-done", status: .completed)))
        XCTAssertEqual(store.primaryResolutionAction, .none)
        XCTAssertFalse(store.performPrimaryResolutionAction())
    }

    @MainActor
    func test_perform_primary_resolution_focuses_contract_and_contract_tab() {
        let store = SteadStore()
        store.selectedTab = .sessions
        store.apply(.contractUpsert(makeContract(id: "c-attn", status: .failed)))

        XCTAssertTrue(store.performPrimaryResolutionAction())
        XCTAssertEqual(store.focusedContractId, "c-attn")
        XCTAssertEqual(store.selectedTab, .contracts)
    }

    @MainActor
    func test_keyboard_shortcuts_switch_tabs() {
        let store = SteadStore()
        store.selectedTab = .sessions

        XCTAssertTrue(store.handleKeyboardShortcut(.showContracts))
        XCTAssertEqual(store.selectedTab, .contracts)

        XCTAssertTrue(store.handleKeyboardShortcut(.showSessions))
        XCTAssertEqual(store.selectedTab, .sessions)
    }

    @MainActor
    func test_keyboard_shortcut_resolve_primary_routes_to_resolution_flow() {
        let store = SteadStore()
        store.selectedTab = .sessions
        store.apply(.contractUpsert(makeContract(id: "c-1", status: .failed)))

        XCTAssertTrue(store.handleKeyboardShortcut(.resolvePrimary))
        XCTAssertEqual(store.focusedContractId, "c-1")
        XCTAssertEqual(store.selectedTab, .contracts)
    }

    @MainActor
    func test_cli_ui_vocabulary_alignment_for_shared_families() {
        XCTAssertEqual(
            SteadStore.Tab.allCases.map(\.cliFamilyName),
            ["contract", "session"]
        )
    }

    @MainActor
    func test_cli_ui_order_alignment_for_sidebar_tabs() {
        XCTAssertEqual(
            SteadStore.Tab.allCases.map(\.label),
            ["Contracts", "Sessions"]
        )
    }

    @MainActor
    func test_contract_empty_state_guidance_uses_grouped_cli_surface() {
        let hint = ContractListView.emptyStateHint
        XCTAssertTrue(hint.contains("stead contract create"))
        XCTAssertFalse(hint.contains("stead run"))
    }

    private func makeContract(id: String, status: ContractStatus) -> ContractItem {
        ContractItem(
            id: id,
            task: "task-\(id)",
            verification: "echo ok",
            status: status,
            createdAt: "2026-02-16T00:00:00Z",
            completedAt: nil,
            output: nil,
            owner: nil,
            blockedBy: [],
            blocks: []
        )
    }
}
