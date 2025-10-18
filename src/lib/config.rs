// src/lib/config.rs

// struct type to represent the application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
}

// methods to build the configuration
impl AppConfig {
    // constructor for AppConfig with JWT secret
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    // Get JWT secret as bytes for key generation
    pub fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }
}

// implement the default trait for AppConfig (for testing)
impl Default for AppConfig {
    // provide a default implementation for testing
    fn default() -> Self {
        Self::new("test-secret-key-for-development".to_string())
    }
}
