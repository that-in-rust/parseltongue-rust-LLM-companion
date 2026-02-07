<?php
// v151 Bug Reproduction: PHP Qualified Names with \
// PHP uses \ for namespace separators
// Expected: Edges should be created, keys should be properly sanitized

namespace MyApp\Services;

use MyApp\Models\User;
use MyApp\Events\Publisher;

class UserService
{
    public function createUser(): User
    {
        // Edge 1: Fully qualified class instantiation
        $user = new \MyApp\Models\User();

        // Edge 2: Global namespace class
        $date = new \DateTime();

        // Edge 3: Qualified static method call
        \MyApp\Utils\Validator::validate($user);

        // Edge 4: Nested namespace access
        $config = \MyApp\Config\Settings::DATABASE;

        $this->processUser($user);
        return $user;
    }

    private function processUser(User $user): void
    {
        // Edge 5: Fully qualified function call
        \MyApp\Events\Publisher::publish("user.created", $user);

        // Edge 6: PHP built-in with global namespace
        \error_log("Processing user: " . $user->getName());

        // Edge 7: Qualified exception
        if (!$user->isValid()) {
            throw new \InvalidArgumentException("Invalid user");
        }
    }
}

namespace MyApp\Models;

class User
{
    private string $name = "";

    public function getName(): string
    {
        return $this->name;
    }

    public function isValid(): bool
    {
        return !empty($this->name);
    }
}

namespace MyApp\Utils;

class Validator
{
    public static function validate(object $obj): bool
    {
        return true;
    }
}

namespace MyApp\Events;

class Publisher
{
    public static function publish(string $event, object $data): void
    {
        echo "Publishing: $event\n";
    }
}

namespace MyApp\Config;

class Settings
{
    public const DATABASE = "mysql://localhost/myapp";
}
