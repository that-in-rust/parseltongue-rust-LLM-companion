import Foundation

// MARK: - Data Models

struct User {
    let id: UUID
    let name: String
    let email: String
}

enum UserRole {
    case admin
    case moderator
    case user
}

// MARK: - Protocols

protocol UserRepository {
    func fetch(id: UUID) async throws -> User
    func save(_ user: User) async throws
}

// MARK: - Implementations

class InMemoryUserRepository: UserRepository {
    private var users: [UUID: User] = [:]

    func fetch(id: UUID) async throws -> User {
        guard let user = users[id] else {
            throw RepositoryError.notFound
        }
        return user
    }

    func save(_ user: User) async throws {
        users[user.id] = user
    }
}

// MARK: - Utility Functions

func validateEmail(_ email: String) -> Bool {
    let regex = try! NSRegularExpression(pattern: "^[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}$")
    return regex.firstMatch(in: email, range: NSRange(email.startIndex..., in: email)) != nil
}

func generateUserKey(for user: User) -> String {
    return "user:\(user.id.uuidString)"
}
