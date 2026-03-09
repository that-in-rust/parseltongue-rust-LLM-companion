// Test Swift file for entity extraction
import Foundation

// Function
func calculateSum(a: Int, b: Int) -> Int {
    return a + b
}

// Class
class UserManager {
    var users: [String] = []

    func addUser(name: String) {
        users.append(name)
    }
}

// Struct
struct Point {
    var x: Double
    var y: Double

    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

// Enum
enum Direction {
    case north
    case south
    case east
    case west
}

// Protocol
protocol Drawable {
    func draw()
}
