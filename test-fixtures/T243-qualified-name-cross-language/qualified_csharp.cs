// C# qualified names using .
using System;
using System.Collections.Generic;
using MyApp.Services;

namespace MyApp.Controllers {
    public class UserController {
        public void Index() {
            var list = new List<string>();
            var service = new MyApp.Services.UserService();
        }
    }
}
