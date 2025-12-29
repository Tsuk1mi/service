import SwiftUI
import RimskiyShared
import ComposeApp

@main
struct iosApp: App {
    var body: some Scene {
        WindowGroup {
            ComposeView()
        }
    }
}

struct ComposeView: UIViewControllerRepresentable {
    func makeUIViewController(context: Context) -> UIViewController {
        // Используем Kotlin функцию из shared модуля
        MainViewControllerKt.MainViewController()
    }
    
    func updateUIViewController(_ uiViewController: UIViewController, context: Context) {
        // Обновление не требуется, так как UI управляется Compose
    }
}
