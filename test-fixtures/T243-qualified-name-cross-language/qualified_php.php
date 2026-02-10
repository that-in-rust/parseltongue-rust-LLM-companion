<?php
// PHP qualified names using \
namespace App\Controllers;

use App\Models\User;
use App\Services\AuthService;

class UserController {
    public function index() {
        $user = new \App\Models\User();
        $auth = new AuthService();
    }
}
