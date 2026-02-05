import SwiftUI

struct SessionListView: View {
    @ObservedObject var store: SteadStore
    @State private var selectedSession: SessionItem?

    var body: some View {
        HSplitView {
            sessionList
                .frame(minWidth: 300)
            sessionDetail
                .frame(minWidth: 300)
        }
    }

    private var sessionList: some View {
        Group {
            if store.sessions.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "bubble.left.and.bubble.right")
                        .font(.system(size: 40))
                        .foregroundStyle(.secondary)
                    Text("No Sessions")
                        .font(.title3)
                        .foregroundStyle(.secondary)
                    Text("Sessions from Claude, Codex, and OpenCode appear here")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(selection: Binding(
                    get: { selectedSession?.id },
                    set: { id in selectedSession = store.sessions.first { $0.id == id } }
                )) {
                    ForEach(store.sessionsByCli, id: \.0) { cli, items in
                        Section(cli.rawValue) {
                            ForEach(items) { session in
                                SessionRow(session: session)
                                    .tag(session.id)
                            }
                        }
                    }
                }
                .listStyle(.inset)
            }
        }
    }

    private var sessionDetail: some View {
        Group {
            if let session = selectedSession {
                SessionDetailView(session: session)
            } else {
                Text("Select a session")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }
}

// MARK: - Session Row

struct SessionRow: View {
    let session: SessionItem

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: session.cli.icon)
                .foregroundStyle(.blue)
                .frame(width: 20)

            VStack(alignment: .leading, spacing: 2) {
                Text(session.title)
                    .lineLimit(1)
                    .font(.body)

                HStack(spacing: 6) {
                    Text(session.projectPath.components(separatedBy: "/").last ?? session.projectPath)
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    if let branch = session.gitBranch {
                        Label(branch, systemImage: "arrow.triangle.branch")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 2) {
                Text("\(session.messageCount) msgs")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(session.lastModified)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Session Detail

struct SessionDetailView: View {
    let session: SessionItem

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                HStack {
                    Image(systemName: session.cli.icon)
                        .font(.title2)
                        .foregroundStyle(.blue)
                    VStack(alignment: .leading) {
                        Text(session.title)
                            .font(.title2.bold())
                        Text(session.cli.rawValue)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }

                Divider()

                DetailSection(title: "Project") {
                    Text(session.projectPath)
                        .font(.body.monospaced())
                        .textSelection(.enabled)
                }

                if let branch = session.gitBranch {
                    DetailSection(title: "Branch") {
                        Label(branch, systemImage: "arrow.triangle.branch")
                    }
                }

                DetailSection(title: "Messages") {
                    Text("\(session.messageCount)")
                }

                DetailSection(title: "Created") {
                    Text(session.created)
                        .font(.caption.monospaced())
                }

                DetailSection(title: "Last Modified") {
                    Text(session.lastModified)
                        .font(.caption.monospaced())
                }

                DetailSection(title: "Session ID") {
                    Text(session.id)
                        .font(.caption.monospaced())
                        .textSelection(.enabled)
                }
            }
            .padding()
        }
    }
}
