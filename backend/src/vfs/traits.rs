use async_trait::async_trait;
use serde::{Deserialize, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Custom serialization for SystemTime to formatted string
fn serialize_system_time<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    
    // Convert to formatted string: yyyy-MM-dd HH:mm:ss
    let datetime = chrono::DateTime::from_timestamp(secs as i64, nanos)
        .unwrap_or_default();
    let formatted = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&formatted)
}

fn serialize_optional_system_time<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        Some(t) => serialize_system_time(t, serializer),
        None => serializer.serialize_none(),
    }
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    #[serde(serialize_with = "serialize_system_time")]
    pub modified: SystemTime,
    #[serde(serialize_with = "serialize_optional_system_time")]
    pub created: Option<SystemTime>,
    #[serde(serialize_with = "serialize_optional_system_time")]
    pub accessed: Option<SystemTime>,
    pub permissions: Option<FilePermissions>,
}

/// File permissions (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

/// Directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    #[serde(serialize_with = "serialize_system_time")]
    pub modified: SystemTime,
    #[serde(serialize_with = "serialize_optional_system_time")]
    pub created: Option<SystemTime>,
}

/// Directory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
}

/// File system abstraction trait
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// List directory contents
    async fn list_dir(&self, path: &Path) -> crate::error::Result<Vec<FileEntry>>;

    /// Get file/directory metadata
    async fn metadata(&self, path: &Path) -> crate::error::Result<FileMetadata>;

    /// Read file content
    async fn read_file(&self, path: &Path) -> crate::error::Result<Vec<u8>>;

    /// Read file content as stream (for large files)
    async fn read_file_stream(
        &self,
        path: &Path,
        offset: Option<u64>,
        length: Option<u64>,
    ) -> crate::error::Result<Vec<u8>>;

    /// Write file content
    async fn write_file(&self, path: &Path, content: &[u8]) -> crate::error::Result<()>;

    /// Create directory and all parent directories
    async fn create_dir_all(&self, path: &Path) -> crate::error::Result<()>;

    /// Delete file or directory
    async fn delete(&self, path: &Path) -> crate::error::Result<()>;

    /// Rename/move file or directory
    async fn rename(&self, from: &Path, to: &Path) -> crate::error::Result<()>;

    /// Copy file or directory
    async fn copy(&self, from: &Path, to: &Path) -> crate::error::Result<()>;

    /// Search files by name pattern
    async fn search(
        &self,
        path: &Path,
        pattern: &str,
        recursive: bool,
    ) -> crate::error::Result<Vec<SearchResult>>;

    /// Get directory statistics
    async fn dir_stats(&self, path: &Path) -> crate::error::Result<DirStats>;

    /// Get MIME type for file
    fn mime_type(&self, path: &Path) -> String {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    }
}
