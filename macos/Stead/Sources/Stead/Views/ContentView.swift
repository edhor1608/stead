import SwiftUI

struct ContentView: View {
    @ObservedObject var store: SteadStore

    var body: some View {
        NavigationSplitView {
            List(selection: $store.selectedTab) {
                Label("Contracts", systemImage: "doc.text")
                    .tag(SteadStore.Tab.contracts)
                Label("Sessions", systemImage: "bubble.left.and.bubble.right")
                    .tag(SteadStore.Tab.sessions)
            }
            .listStyle(.sidebar)
            .frame(minWidth: 160)
        } detail: {
            ZStack(alignment: .topLeading) {
                switch store.selectedTab {
                case .contracts:
                    ContractListView(store: store)
                case .sessions:
                    SessionListView(store: store)
                }

                if let error = store.errorMessage, !error.isEmpty {
                    Text(error)
                        .font(.caption)
                        .foregroundStyle(.white)
                        .padding(8)
                        .background(.red.opacity(0.85))
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                        .padding()
                        .textSelection(.enabled)
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(action: { store.refresh() }) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .keyboardShortcut("r", modifiers: .command)
            }
        }
        .onAppear { store.refresh() }
    }
}
