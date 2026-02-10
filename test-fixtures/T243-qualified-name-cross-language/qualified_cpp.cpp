// C++ qualified names using ::
#include <vector>
#include <string>

namespace myapp {
    namespace services {
        class UserService {
        public:
            void process() {}
        };
    }
}

int main() {
    std::vector<int> data;
    myapp::services::UserService service;
    service.process();
    return 0;
}
