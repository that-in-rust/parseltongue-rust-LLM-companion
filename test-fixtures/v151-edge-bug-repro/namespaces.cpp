// v151 Bug Reproduction: C++ Qualified Names with ::
// This file tests the qualified name bug where :: breaks ISGL1 key parsing

#include <iostream>
#include <vector>
#include <string>

namespace MyApp {
namespace Services {

class UserService {
public:
    void createUser() {
        // Edge 1: std:: namespace prefix
        std::vector<std::string> users;

        // Edge 2: Method call on std:: container
        users.push_back("user1");

        // Edge 3: Nested std:: call
        std::cout << "Created user" << std::endl;

        // Edge 4: Qualified method call within namespace
        processUser("test");
    }

private:
    void processUser(const std::string& name) {
        // Edge 5: Constructor with std:: namespace
        std::string processed = name;

        // Edge 6: std::cout usage
        std::cout << processed << std::endl;
    }
};

} // namespace Services
} // namespace MyApp

// Edge 7: Fully qualified function call
void globalFunction() {
    ::MyApp::Services::UserService service;
    service.createUser();
}
