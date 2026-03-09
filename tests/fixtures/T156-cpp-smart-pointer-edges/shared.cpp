void share() {
    auto config = std::make_shared<Config>();
    auto manager = std::make_shared<Manager>(config);
}
