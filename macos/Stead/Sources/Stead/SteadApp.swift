import SwiftUI

@main
struct SteadApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var store = SteadStore()

    var body: some Scene {
        MenuBarExtra("Stead", systemImage: "square.stack.3d.up") {
            MenuBarView(store: store)
        }
        .menuBarExtraStyle(.window)

        Window("Stead Control Room", id: "main") {
            ContentView(store: store)
                .frame(minWidth: 700, minHeight: 500)
        }
        .defaultSize(width: 900, height: 650)
    }
}
