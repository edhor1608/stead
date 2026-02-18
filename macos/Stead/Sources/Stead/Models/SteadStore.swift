import Foundation
import SwiftUI
import AppKit
import UserNotifications

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
    let projectPath: String
    let task: String
    let verification: String
    let status: ContractStatus
    let createdAt: String
    let completedAt: String?
    let output: String?
    let owner: String?
    let blockedBy: [String]
    let blocks: [String]

    init(ffi: FfiContract) {
        self.id = ffi.id
        self.projectPath = ffi.projectPath
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
    @Published var errorMessage: String?

    private var watchers: [FileWatcher] = []
    private var lastStatusByContractId: [String: ContractStatus] = [:]
    private var notificationAuthRequested = false

    private let defaults = UserDefaults.standard
    private let rootsKey = "stead.workspaceRoots"
    private let ownerKey = "stead.ownerName"
    private let maxDepthKey = "stead.discoveryMaxDepth"

    private(set) var workspaceRoots: [String]
    private(set) var discoveryMaxDepth: Int
    private(set) var ownerName: String

    enum Tab {
        case contracts
        case sessions
    }

    init() {
        self.workspaceRoots = defaults.stringArray(forKey: rootsKey) ?? SteadStore.defaultWorkspaceRoots()
        self.discoveryMaxDepth = defaults.object(forKey: maxDepthKey) as? Int ?? 4
        self.ownerName = defaults.string(forKey: ownerKey) ?? NSFullUserName()
        requestNotificationAuthorizationIfNeeded()
    }

    func refresh() {
        loadContracts()
        loadSessions()
    }

    func loadContracts() {
        let roots = workspaceRoots
        let maxDepth = discoveryMaxDepth

        Task.detached { [weak self] in
            let projectPaths = Self.discoverSteadProjects(roots: roots, maxDepth: maxDepth)

            var newContracts: [ContractItem] = []
            var errors: [String] = []
            for projectPath in projectPaths {
                do {
                    let ffiContracts = try listContracts(cwd: projectPath)
                    newContracts.append(contentsOf: ffiContracts.map { ContractItem(ffi: $0) })
                } catch {
                    // Keep going; one broken DB shouldn't blank the whole Control Room.
                    errors.append("Failed to load contracts for \(projectPath): \(error.localizedDescription)")
                }
            }

            await MainActor.run {
                guard let self else { return }
                // Fire notifications on status transitions before swapping state.
                self.postNotificationsIfNeeded(next: newContracts)
                self.contracts = newContracts
                self.errorMessage = errors.isEmpty ? nil : errors.joined(separator: "\n")

                // Keep file watchers aligned with what we're displaying.
                self.rebuildWatchers(projectPaths: projectPaths)
            }
        }
    }

    func loadSessions() {
        let ffiSessions = listSessions(cliFilter: nil, project: nil, limit: 50)
        sessions = ffiSessions.map { SessionItem(ffi: $0) }
    }

    func claim(contract: ContractItem) {
        Task.detached { [contractId = contract.id, ownerName = ownerName, projectPath = contract.projectPath] in
            do {
                _ = try claimContract(id: contractId, owner: ownerName, cwd: projectPath)
                await MainActor.run { [weak self] in self?.refresh() }
            } catch {
                await MainActor.run { [weak self] in
                    self?.errorMessage = "Failed to claim contract: \(error.localizedDescription)"
                }
            }
        }
    }

    func cancel(contract: ContractItem) {
        Task.detached { [contractId = contract.id, projectPath = contract.projectPath] in
            do {
                _ = try cancelContract(id: contractId, cwd: projectPath)
                await MainActor.run { [weak self] in self?.refresh() }
            } catch {
                await MainActor.run { [weak self] in
                    self?.errorMessage = "Failed to cancel contract: \(error.localizedDescription)"
                }
            }
        }
    }

    func verify(contract: ContractItem) {
        Task.detached { [contractId = contract.id, projectPath = contract.projectPath] in
            do {
                _ = try verifyContract(id: contractId, cwd: projectPath)
                await MainActor.run { [weak self] in self?.refresh() }
            } catch {
                await MainActor.run { [weak self] in
                    self?.errorMessage = "Failed to verify contract: \(error.localizedDescription)"
                }
            }
        }
    }

    func openProject(for contract: ContractItem) {
        ContextRestore.openProject(path: contract.projectPath)
    }

    /// Contracts grouped by attention priority
    var contractsByPriority: [(String, [ContractItem])] {
        let sorted = contracts.sorted { $0.status.attentionPriority < $1.status.attentionPriority }
        let grouped = Dictionary(grouping: sorted) { $0.status }
        return ContractStatus.allCases
            .sorted { $0.attentionPriority < $1.attentionPriority }
            .compactMap { status in
                guard let items = grouped[status], !items.isEmpty else { return nil }
                return (status.rawValue, items)
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

    private static func defaultWorkspaceRoots() -> [String] {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        let candidates = [
            "\(home)/repos",
            "\(home)/Projects",
            home,
        ]
        return candidates.filter {
            var isDir: ObjCBool = false
            return FileManager.default.fileExists(atPath: $0, isDirectory: &isDir) && isDir.boolValue
        }
    }

    private static func discoverSteadProjects(roots: [String], maxDepth: Int) -> [String] {
        var out: [String] = []
        var seen: Set<String> = []

        for root in roots {
            Self.walkDir(path: root, depth: maxDepth, out: &out, seen: &seen)
        }

        return out.sorted()
    }

    private static func walkDir(path: String, depth: Int, out: inout [String], seen: inout Set<String>) {
        guard depth >= 0 else { return }

        // If this directory is a Stead project, record it and do not recurse further.
        let dbPath = (path as NSString).appendingPathComponent(".stead/stead.db")
        let legacyPath = (path as NSString).appendingPathComponent(".stead/contracts.jsonl")
        if FileManager.default.fileExists(atPath: dbPath) || FileManager.default.fileExists(atPath: legacyPath) {
            if !seen.contains(path) {
                seen.insert(path)
                out.append(path)
            }
            return
        }

        guard depth > 0 else { return }

        let denylist: Set<String> = [".git", "node_modules", "target", ".build", "dist", "build", ".venv", "venv", "DerivedData"]
        guard let entries = try? FileManager.default.contentsOfDirectory(atPath: path) else { return }

        for name in entries {
            if denylist.contains(name) { continue }
            let child = (path as NSString).appendingPathComponent(name)

            var isDir: ObjCBool = false
            if FileManager.default.fileExists(atPath: child, isDirectory: &isDir), isDir.boolValue {
                Self.walkDir(path: child, depth: depth - 1, out: &out, seen: &seen)
            }
        }
    }

    private func rebuildWatchers(projectPaths: [String]) {
        watchers.removeAll()

        for projectPath in projectPaths {
            let steadDir = (projectPath as NSString).appendingPathComponent(".stead")
            let dbPath = (steadDir as NSString).appendingPathComponent("stead.db")
            let walPath = dbPath + "-wal"
            let shmPath = dbPath + "-shm"

            for path in [steadDir, dbPath, walPath, shmPath] {
                let watcher = FileWatcher(path: path) { [weak self] in
                    Task { @MainActor in
                        self?.refresh()
                    }
                }
                if let watcher = watcher {
                    watchers.append(watcher)
                }
            }
        }
    }

    private func requestNotificationAuthorizationIfNeeded() {
        guard !notificationAuthRequested else { return }
        notificationAuthRequested = true

        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in
            // Ignore; we'll just not post if permission denied.
        }
    }

    private func postNotificationsIfNeeded(next: [ContractItem]) {
        let center = UNUserNotificationCenter.current()

        var nextStatusById: [String: ContractStatus] = [:]
        for c in next {
            nextStatusById[c.id] = c.status
        }

        for c in next {
            let prev = lastStatusByContractId[c.id]
            if prev == nil {
                continue
            }

            // Notify on transitions into terminal outcome states.
            if (c.status == .completed || c.status == .failed),
               prev != c.status
            {
                let content = UNMutableNotificationContent()
                let projectName = URL(fileURLWithPath: c.projectPath).lastPathComponent
                content.title = "\(projectName): \(c.status.rawValue)"
                content.body = c.task
                content.sound = .default
                content.userInfo = [
                    "contractId": c.id,
                    "projectPath": c.projectPath,
                ]

                let request = UNNotificationRequest(
                    identifier: "contract-\(c.id)-\(c.status.rawValue)",
                    content: content,
                    trigger: nil
                )

                center.add(request)
            }
        }

        lastStatusByContractId = nextStatusById
    }
}

final class FileWatcher {
    private let fd: Int32
    private let source: DispatchSourceFileSystemObject

    init?(path: String, onChange: @escaping () -> Void) {
        fd = open(path, O_EVTONLY)
        if fd < 0 {
            return nil
        }

        let queue = DispatchQueue(label: "stead.filewatcher")
        source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete, .extend, .attrib, .link, .revoke],
            queue: queue
        )

        // Debounce bursts (SQLite WAL can be chatty).
        var pending = false
        source.setEventHandler {
            if pending { return }
            pending = true
            queue.asyncAfter(deadline: .now() + 0.25) {
                pending = false
                onChange()
            }
        }

        source.setCancelHandler { [fd] in
            close(fd)
        }

        source.resume()
    }

    deinit {
        source.cancel()
    }
}

enum ContextRestore {
    static func openProject(path: String) {
        let url = URL(fileURLWithPath: path)
        NSWorkspace.shared.open(url)

        // Terminal
        let terminalUrl = URL(fileURLWithPath: "/System/Applications/Utilities/Terminal.app")
        if FileManager.default.fileExists(atPath: terminalUrl.path) {
            NSWorkspace.shared.open([url], withApplicationAt: terminalUrl, configuration: NSWorkspace.OpenConfiguration(), completionHandler: nil)
        }

        // VS Code (best-effort)
        let vscodeUrl = URL(fileURLWithPath: "/Applications/Visual Studio Code.app")
        if FileManager.default.fileExists(atPath: vscodeUrl.path) {
            NSWorkspace.shared.open([url], withApplicationAt: vscodeUrl, configuration: NSWorkspace.OpenConfiguration(), completionHandler: nil)
        }
    }
}
