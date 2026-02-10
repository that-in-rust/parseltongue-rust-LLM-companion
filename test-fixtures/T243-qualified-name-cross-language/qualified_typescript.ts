// TypeScript qualified names using .
import { UserService } from './services/UserService';

namespace MyApp.Controllers {
    export class UserController {
        private service: MyApp.Services.UserService;

        public index(): void {
            const data: Array<string> = [];
            this.service = new UserService();
        }
    }
}
