# v151 Bug Reproduction: Python Module Paths
# Python uses . for module paths (not :: so LESS affected)
# Included for completeness - Python should work correctly

import os
import sys
from typing import Optional, List, Dict
from dataclasses import dataclass

# Direct module access patterns
import collections.abc
import urllib.parse
import json.decoder


@dataclass
class User:
    name: str
    email: str = ""


class UserService:
    def create_user(self) -> User:
        # Edge 1: Qualified module function
        path = os.path.join("/tmp", "users")

        # Edge 2: Nested module access
        parsed = urllib.parse.urlparse("http://example.com")

        # Edge 3: sys module access
        sys.stdout.write("Creating user\n")

        # Edge 4: collections.abc usage
        isinstance([], collections.abc.Sequence)

        user = User(name="test")
        self._process_user(user)
        return user

    def _process_user(self, user: User) -> None:
        # Edge 5: json module nested access
        try:
            json.decoder.JSONDecodeError
        except:
            pass

        # Edge 6: os.path nested call
        os.path.exists("/tmp")

        # Edge 7: typing module (if accessed directly)
        annotations: Dict[str, str] = {}

        print(f"Processing: {user.name}")


def main():
    service = UserService()
    user = service.create_user()

    # Edge 8: sys.exit
    sys.exit(0)


if __name__ == "__main__":
    main()
