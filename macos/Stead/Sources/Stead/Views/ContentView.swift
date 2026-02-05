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
            switch store.selectedTab {
            case .contracts:
                ContractListView(store: store)
            case .sessions:
                SessionListView(store: store)
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
