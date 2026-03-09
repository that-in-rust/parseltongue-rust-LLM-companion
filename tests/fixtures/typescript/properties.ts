// Test fixture for TypeScript property access detection
// REQ-TYPESCRIPT-002.0

class ConfigReader {
    read(config: Config) {
        // Simple property access
        const setting = config.enabled;
        const value = config.maxSize;

        // Chained property access
        const nested = config.database.host;
    }

    write(config: Config) {
        // Property assignment
        config.enabled = true;
        config.maxSize = 100;
    }
}

interface Config {
    enabled: boolean;
    maxSize: number;
    database: DatabaseConfig;
}

interface DatabaseConfig {
    host: string;
    port: number;
}
