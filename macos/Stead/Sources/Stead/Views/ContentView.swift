import SwiftUI

struct ContentView: View {
    @ObservedObject var store: SteadStore

    var body: some View {
        NavigationSplitView {
            List(selection: $store.selectedTab) {
                ForEach(SteadStore.Tab.allCases, id: \.self) { tab in
                    Label(tab.label, systemImage: tab.icon)
                        .tag(tab)
                }
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
                Button(action: { _ = store.handleKeyboardShortcut(.refresh) }) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .keyboardShortcut("r", modifiers: .command)
            }
        }
        .onAppear { store.refresh() }
    }
}
