import SwiftUI

/// The shared root view for iPhone, iPad, and Mac. Kept deliberately plain —
/// no brand colors or type have been specified yet, so this leans on system
/// semantic styles until a visual identity lands.
public struct ContentView: View {
    @State private var projects: [Project] = []

    public init() {}

    public var body: some View {
        NavigationStack {
            Group {
                if projects.isEmpty {
                    ContentUnavailableView(
                        "No Projects Yet",
                        systemImage: "square.on.square.dashed",
                        description: Text("Start a new project to turn an idea into a pattern.")
                    )
                } else {
                    List(projects) { project in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(project.name)
                                .font(.headline)
                            Text("\(project.pieces.count) pieces")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.vertical, 4)
                    }
                }
            }
            .navigationTitle("Patruin")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button(action: addPlaceholderProject) {
                        Label("New Project", systemImage: "plus")
                    }
                }
            }
        }
    }

    private func addPlaceholderProject() {
        projects.append(Project(name: "Untitled Project"))
    }
}
