pub mod config_manager;

// Re-export commonly used types
pub use config_manager::{read_config_file, write_config_file, ConfigManager};
