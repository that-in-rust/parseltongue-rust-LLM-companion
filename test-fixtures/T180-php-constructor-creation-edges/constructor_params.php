<?php
class Factory {
    public function make() {
        $db = new Database("localhost", 3306);
        $cache = new RedisCache("127.0.0.1", 6379);
    }
}
