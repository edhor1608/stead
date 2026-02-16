import Foundation
import SwiftUI

// MARK: - App-level types wrapping FFI

enum ContractStatus: String, CaseIterable {
    case pending = "Pending"
    case ready = "Ready"
    case claimed = "Claimed"
    case executing = "Executing"
    case verifying = "Verifying"
    case completed = "Completed"
    case failed = "Failed"
    case rollingBack = "Rolling Back"
    case rolledBack = "Rolled Back"
    case cancelled = "Cancelled"

    var icon: String {
        switch self {
        case .pending: return "clock"
        case .ready: return "tray.full"
        case .claimed: return "person.fill"
        case .executing: return "bolt.fill"
        case .verifying: return "magnifyingglass"
        case .completed: return "checkmark.circle.fill"
        case .failed: return "xmark.circle.fill"
        case .rollingBack: return "arrow.uturn.backward"
        case .rolledBack: return "arrow.uturn.backward.circle"
        case .cancelled: return "minus.circle"
        }
    }

    var color: Color {
        switch self {
        case .pending: return .secondary
        case .ready: return .orange
        case .claimed: return .purple
        case .executing: return .blue
        case .verifying: return .cyan
        case .completed: return .green
        case .failed: return .red
        case .rollingBack: return .yellow
        case .rolledBack: return .secondary
        case .cancelled: return .secondary
        }
    }

    /// Lower number = higher attention priority
    var attentionPriority: Int {
        switch self {
        case .failed: return 0
        case .rollingBack: return 1
        case .verifying: return 2
        case .completed: return 3
        case .executing: return 4
        case .claimed: return 5
        case .ready: return 6
        case .pending: return 7
        case .rolledBack: return 8
        case .cancelled: return 9
        }
    }
}

struct ContractItem: Identifiable {
    let id: String
    let task: String
    let verification: String
    let status: ContractStatus
    let createdAt: String
    let completedAt: String?
    let output: String?
    let owner: String?
    let blockedBy: [String]
    let blocks: [String]

    init(
        id: String,
        task: String,
        verification: String,
        status: ContractStatus,
        createdAt: String,
        completedAt: String?,
        output: String?,
        owner: String?,
        blockedBy: [String],
        blocks: [String]
    ) {
        self.id = id
        self.task = task
        self.verification = verification
        self.status = status
        self.createdAt = createdAt
        self.completedAt = completedAt
        self.output = output
        self.owner = owner
        self.blockedBy = blockedBy
        self.blocks = blocks
    }

    init(ffi: FfiContract) {
        self.id = ffi.id
        self.task = ffi.task
        self.verification = ffi.verification
        self.status = {
            switch ffi.status {
            case .pending: return .pending
            case .ready: return .ready
            case .claimed: return .claimed
            case .executing: return .executing
            case .verifying: return .verifying
            case .completed: return .completed
            case .failed: return .failed
            case .rollingBack: return .rollingBack
            case .rolledBack: return .rolledBack
            case .cancelled: return .cancelled
            }
        }()
        self.createdAt = ffi.createdAt
        self.completedAt = ffi.completedAt
        self.output = ffi.output
        self.owner = ffi.owner
        self.blockedBy = ffi.blockedBy
        self.blocks = ffi.blocks
    }
}

enum CliType: String, CaseIterable {
    case claude = "Claude"
    case codex = "Codex"
    case openCode = "OpenCode"
    case universal = "Universal"

    var icon: String {
        switch self {
        case .claude: return "brain"
        case .codex: return "terminal"
        case .openCode: return "chevron.left.forwardslash.chevron.right"
        case .universal: return "globe"
        }
    }
}

struct SessionItem: Identifiable {
    let id: String
    let cli: CliType
    let projectPath: String
    let title: String
    let created: String
    let lastModified: String
    let messageCount: UInt32
    let gitBranch: String?

    init(
        id: String,
        cli: CliType,
        projectPath: String,
        title: String,
        created: String,
        lastModified: String,
        messageCount: UInt32,
        gitBranch: String?
    ) {
        self.id = id
        self.cli = cli
        self.projectPath = projectPath
        self.title = title
        self.created = created
        self.lastModified = lastModified
        self.messageCount = messageCount
        self.gitBranch = gitBranch
    }

    init(ffi: FfiSessionSummary) {
        self.id = ffi.id
        self.cli = {
            switch ffi.cli {
            case .claude: return .claude
            case .codex: return .codex
            case .openCode: return .openCode
            case .universal: return .universal
            }
        }()
        self.projectPath = ffi.projectPath
        self.title = ffi.title
        self.created = ffi.created
        self.lastModified = ffi.lastModified
        self.messageCount = ffi.messageCount
        self.gitBranch = ffi.gitBranch
    }
}

// MARK: - Store

@MainActor
class SteadStore: ObservableObject {
    @Published var contracts: [ContractItem] = []
    @Published var sessions: [SessionItem] = []
    @Published var selectedTab: Tab = .contracts
    @Published var focusedContractId: String?
    @Published var errorMessage: String?

    enum Tab {
        case contracts
        case sessions
    }

    enum MenuBarState {
        case idle
        case running
        case attention
        case decision
    }

    enum PrimaryResolutionAction: Equatable {
        case none
        case resolveDecision(contractId: String)
        case resolveAnomaly(contractId: String)
    }

    enum Event {
        case contractSnapshot([ContractItem])
        case contractUpsert(ContractItem)
        case contractRemoved(id: String)
        case sessionSnapshot([SessionItem])
        case sessionUpsert(SessionItem)
        case sessionRemoved(id: String)
    }

    enum KeyboardShortcut {
        case showContracts
        case showSessions
        case resolvePrimary
        case refresh
    }

    func refresh() {
        loadContracts()
        loadSessions()
    }

    func loadContracts() {
        let cwd = FileManager.default.currentDirectoryPath
        do {
            let ffiContracts = try listContracts(cwd: cwd)
            apply(.contractSnapshot(ffiContracts.map { ContractItem(ffi: $0) }))
            errorMessage = nil
        } catch {
            contracts = []
            errorMessage = "Failed to load contracts: \(error.localizedDescription)"
        }
    }

    func loadSessions() {
        let ffiSessions = listSessions(cliFilter: nil, project: nil, limit: 50)
        apply(.sessionSnapshot(ffiSessions.map { SessionItem(ffi: $0) }))
    }

    func apply(_ event: Event) {
        switch event {
        case let .contractSnapshot(items):
            contracts = items.sorted(by: Self.contractSort)
            if let focused = focusedContractId,
               contracts.contains(where: { $0.id == focused }) == false
            {
                focusedContractId = nil
            }
        case let .contractUpsert(item):
            upsertContract(item)
        case let .contractRemoved(id):
            contracts.removeAll { $0.id == id }
            if focusedContractId == id {
                focusedContractId = nil
            }
        case let .sessionSnapshot(items):
            sessions = items.sorted(by: Self.sessionSort)
        case let .sessionUpsert(item):
            upsertSession(item)
        case let .sessionRemoved(id):
            sessions.removeAll { $0.id == id }
        }
    }

    private func upsertContract(_ item: ContractItem) {
        if let index = contracts.firstIndex(where: { $0.id == item.id }) {
            contracts[index] = item
        } else {
            contracts.append(item)
        }
        contracts.sort(by: Self.contractSort)
    }

    private func upsertSession(_ item: SessionItem) {
        if let index = sessions.firstIndex(where: { $0.id == item.id }) {
            sessions[index] = item
        } else {
            sessions.append(item)
        }
        sessions.sort(by: Self.sessionSort)
    }

    private static func contractSort(_ lhs: ContractItem, _ rhs: ContractItem) -> Bool {
        let lp = lhs.status.attentionPriority
        let rp = rhs.status.attentionPriority
        if lp != rp {
            return lp < rp
        }
        return lhs.id < rhs.id
    }

    private static func sessionSort(_ lhs: SessionItem, _ rhs: SessionItem) -> Bool {
        if lhs.lastModified != rhs.lastModified {
            return lhs.lastModified > rhs.lastModified
        }
        return lhs.id < rhs.id
    }

    /// Contracts grouped by attention priority
    var contractsByPriority: [(String, [ContractItem])] {
        let sorted = contracts.sorted(by: Self.contractSort)
        let grouped = Dictionary(grouping: sorted) { $0.status }
        return ContractStatus.allCases
            .sorted { $0.attentionPriority < $1.attentionPriority }
            .compactMap { status in
                guard let items = grouped[status], !items.isEmpty else { return nil }
                return (status.rawValue, items)
            }
    }

    var menuBarState: MenuBarState {
        if contracts.contains(where: { $0.status == .verifying }) {
            return .decision
        }
        if contracts.contains(where: {
            $0.status == .failed || $0.status == .rollingBack || $0.status == .rolledBack
        }) {
            return .attention
        }
        if contracts.contains(where: {
            $0.status == .pending
                || $0.status == .ready
                || $0.status == .claimed
                || $0.status == .executing
        }) {
            return .running
        }
        return .idle
    }

    var primaryResolutionAction: PrimaryResolutionAction {
        if let decision = contracts
            .sorted(by: Self.contractSort)
            .first(where: { $0.status == .verifying })
        {
            return .resolveDecision(contractId: decision.id)
        }

        if let anomaly = contracts
            .sorted(by: Self.contractSort)
            .first(where: {
                $0.status == .failed || $0.status == .rollingBack || $0.status == .rolledBack
            })
        {
            return .resolveAnomaly(contractId: anomaly.id)
        }

        return .none
    }

    @discardableResult
    func performPrimaryResolutionAction() -> Bool {
        let contractId: String
        switch primaryResolutionAction {
        case .none:
            return false
        case let .resolveDecision(id), let .resolveAnomaly(id):
            contractId = id
        }

        selectedTab = .contracts
        focusedContractId = contractId
        return true
    }

    @discardableResult
    func handleKeyboardShortcut(_ shortcut: KeyboardShortcut) -> Bool {
        switch shortcut {
        case .showContracts:
            selectedTab = .contracts
            return true
        case .showSessions:
            selectedTab = .sessions
            return true
        case .resolvePrimary:
            return performPrimaryResolutionAction()
        case .refresh:
            refresh()
            return true
        }
    }

    /// Sessions grouped by CLI type
    var sessionsByCli: [(CliType, [SessionItem])] {
        let grouped = Dictionary(grouping: sessions) { $0.cli }
        return CliType.allCases.compactMap { cli in
            guard let items = grouped[cli], !items.isEmpty else { return nil }
            return (cli, items)
        }
    }
}
