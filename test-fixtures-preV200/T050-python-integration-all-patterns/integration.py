from typing import List

class UserService:
    @property
    def config(self):
        return self._config

    async def process_users(self, users: List[User]):
        for user in users:
            name = user.name
            result = await save_user(user)
            logger = Logger()
