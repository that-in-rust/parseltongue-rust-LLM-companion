<?php
class MyClass {
    public function test() {
        parent::doWork();
        self::helper();
        static::factory();
    }
}
