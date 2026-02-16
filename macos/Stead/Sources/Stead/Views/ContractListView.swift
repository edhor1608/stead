import SwiftUI

struct ContractListView: View {
    @ObservedObject var store: SteadStore

    var body: some View {
        HSplitView {
            contractList
                .frame(minWidth: 300)
            contractDetail
                .frame(minWidth: 300)
        }
    }

    private var contractList: some View {
        Group {
            if store.contracts.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "doc.text.magnifyingglass")
                        .font(.system(size: 40))
                        .foregroundStyle(.secondary)
                    Text("No Contracts")
                        .font(.title3)
                        .foregroundStyle(.secondary)
                    Text("Run `stead run` to create contracts")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(selection: Binding(
                    get: { store.focusedContractId },
                    set: { id in store.focusedContractId = id }
                )) {
                    ForEach(store.contractsByPriority, id: \.0) { label, items in
                        Section(label) {
                            ForEach(items) { contract in
                                ContractRow(contract: contract)
                                    .tag(contract.id)
                            }
                        }
                    }
                }
                .listStyle(.inset)
            }
        }
    }

    private var contractDetail: some View {
        Group {
            if let selectedId = store.focusedContractId,
               let contract = store.contracts.first(where: { $0.id == selectedId })
            {
                ContractDetailView(contract: contract)
            } else {
                Text("Select a contract")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }
}

// MARK: - Contract Row

struct ContractRow: View {
    let contract: ContractItem

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: contract.status.icon)
                .foregroundStyle(contract.status.color)
                .frame(width: 20)

            VStack(alignment: .leading, spacing: 2) {
                Text(contract.task)
                    .lineLimit(1)
                    .font(.body)
                Text(contract.id)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Text(contract.status.rawValue)
                .font(.caption)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(contract.status.color.opacity(0.15))
                .clipShape(Capsule())
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Contract Detail

struct ContractDetailView: View {
    let contract: ContractItem

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                HStack {
                    Image(systemName: contract.status.icon)
                        .font(.title2)
                        .foregroundStyle(contract.status.color)
                    Text(contract.status.rawValue)
                        .font(.title2.bold())
                        .foregroundStyle(contract.status.color)
                    Spacer()
                    Text(contract.id)
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                }

                Divider()

                // Task
                DetailSection(title: "Task") {
                    Text(contract.task)
                        .textSelection(.enabled)
                }

                // Verification
                DetailSection(title: "Verification") {
                    Text(contract.verification)
                        .font(.body.monospaced())
                        .textSelection(.enabled)
                }

                // Timestamps
                DetailSection(title: "Created") {
                    Text(contract.createdAt)
                        .font(.caption.monospaced())
                }

                if let owner = contract.owner {
                    DetailSection(title: "Owner") {
                        Text(owner)
                    }
                }

                if let completed = contract.completedAt {
                    DetailSection(title: "Completed") {
                        Text(completed)
                            .font(.caption.monospaced())
                    }
                }

                if !contract.blockedBy.isEmpty {
                    DetailSection(title: "Blocked By") {
                        Text(contract.blockedBy.joined(separator: ", "))
                            .font(.caption.monospaced())
                    }
                }

                if !contract.blocks.isEmpty {
                    DetailSection(title: "Blocks") {
                        Text(contract.blocks.joined(separator: ", "))
                            .font(.caption.monospaced())
                    }
                }

                // Output
                if let output = contract.output, !output.isEmpty {
                    DetailSection(title: "Output") {
                        Text(output)
                            .font(.caption.monospaced())
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(.quaternary)
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .textSelection(.enabled)
                    }
                }
            }
            .padding()
        }
    }
}

struct DetailSection<Content: View>: View {
    let title: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
            content
        }
    }
}
