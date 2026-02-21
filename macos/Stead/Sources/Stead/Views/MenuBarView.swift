import SwiftUI

struct MenuBarView: View {
    @ObservedObject var store: SteadStore
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Summary header
            HStack {
                Text("Stead")
                    .font(.headline)
                Spacer()
                Button(action: { store.refresh() }) {
                    Image(systemName: "arrow.clockwise")
                }
                .buttonStyle(.borderless)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)

            Divider()

            // Contract summary
            if store.contracts.isEmpty {
                Text("No contracts")
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
            } else {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(store.contractsByPriority, id: \.0) { label, items in
                        HStack {
                            statusIcon(for: label)
                            Text(label)
                                .font(.caption)
                            Spacer()
                            Text("\(items.count)")
                                .font(.caption.bold())
                                .foregroundStyle(.secondary)
                        }
                    }
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
            }

            Divider()

            // Sessions summary
            if !store.sessions.isEmpty {
                HStack {
                    Image(systemName: "bubble.left.and.bubble.right")
                        .font(.caption)
                    Text("\(store.sessions.count) sessions")
                        .font(.caption)
                    Spacer()
                }
                .foregroundStyle(.secondary)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)

                Divider()
            }

            // Actions
            if store.primaryResolutionAction != .none {
                Button(primaryResolutionTitle()) {
                    if store.performPrimaryResolutionAction() {
                        openWindow(id: "main")
                    }
                }
                .buttonStyle(.borderless)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)

                Divider()
            }

            Button("Open Control Room") {
                openWindow(id: "main")
            }
            .buttonStyle(.borderless)
            .padding(.horizontal, 12)
            .padding(.vertical, 6)

            Divider()

            Button("Quit Stead") {
                NSApplication.shared.terminate(nil)
            }
            .buttonStyle(.borderless)
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
        }
        .frame(width: 240)
        .onAppear { store.refresh() }
    }

    private func statusIcon(for label: String) -> some View {
        let status = ContractStatus.allCases.first { $0.rawValue == label }
        return Image(systemName: status?.icon ?? "questionmark.circle")
            .foregroundStyle(status?.color ?? .secondary)
            .font(.caption)
    }

    private func primaryResolutionTitle() -> String {
        switch store.primaryResolutionAction {
        case .none:
            return "Resolve"
        case .resolveDecision:
            return "Resolve Decision"
        case .resolveAnomaly:
            return "Resolve Anomaly"
        }
    }
}
