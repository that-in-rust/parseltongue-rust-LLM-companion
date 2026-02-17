<?php
function build() {
    $user = \Models\UserFactory::build();
    $logger = \Utils\LogFactory::create();
}
