use async_trait::async_trait;
use std::path::{Component, Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;

use super::traits::{
    DirStats, FileEntry, FileMetadata, FilePermissions, FileSystem, SearchResult,
};
use crate::error::AppError;

/// Local file system implementation
pub struct LocalFileSystem {
    root: PathBuf,
}

impl LocalFileSystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Resolve path relative to root, preventing path traversal
    fn resolve_path(&self, path: &Path) -> crate::error::Result<PathBuf> {
        // Normalize the path
        let normalized = self.normalize_path(path)?;

        // Check for path traversal
        if normalized
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        {
            return Err(AppError::PathTraversal);
        }

        let resolved = self.root.join(&normalized);

        // Ensure the resolved path is under the root
        if !resolved.starts_with(&self.root) {
            return Err(AppError::PathTraversal);
        }

        Ok(resolved)
    }

    /// Normalize path by removing leading slashes and converting to forward slashes
    fn normalize_path(&self, path: &Path) -> crate::error::Result<PathBuf> {
        let path_str = path.to_string_lossy().to_string();

        // Remove leading slash if present
        let normalized = path_str.trim_start_matches('/');

        // Check for invalid characters
        if normalized.contains("..") {
            return Err(AppError::InvalidPath(path_str));
        }

        // Convert backslashes to forward slashes (for Windows compatibility)
        let normalized = normalized.replace('\\', "/");

        Ok(PathBuf::from(normalized))
    }

    /// Convert absolute path to relative path from root
    fn to_relative(&self, path: &Path) -> crate::error::Result<PathBuf> {
        let relative = path.strip_prefix(&self.root).map_err(|_| {
            AppError::InvalidPath(format!("Path {} is not under root", path.display()))
        })?;
        Ok(relative.to_path_buf())
    }

    /// Get file permissions
    fn get_permissions(&self, metadata: &std::fs::Metadata) -> FilePermissions {
        FilePermissions {
            readable: true, // Assume readable if we can access
            writable: !metadata.permissions().readonly(),
            executable: false, // Would need platform-specific code
        }
    }

    /// Recursively copy directory
    async fn copy_dir_recursive(&self, from: &Path, to: &Path) -> crate::error::Result<()> {
        fs::create_dir_all(to).await?;

        let mut dir = fs::read_dir(from).await?;

        while let Some(entry) = dir.next_entry().await? {
            let from_path = entry.path();
            let to_path = to.join(entry.file_name());

            if from_path.is_dir() {
                Box::pin(self.copy_dir_recursive(&from_path, &to_path)).await?;
            } else {
                fs::copy(&from_path, &to_path).await?;
            }
        }

        Ok(())
    }

    /// Recursively search files
    async fn search_recursive(
        &self,
        path: &Path,
        pattern: &str,
        results: &mut Vec<SearchResult>,
    ) -> crate::error::Result<()> {
        let mut dir = fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Check if name matches pattern (simple glob-like matching)
            if matches_pattern(&name, pattern) {
                let relative_path = self.to_relative(&entry_path)?;
                results.push(SearchResult {
                    path: relative_path,
                    name,
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                });
            }

            // Recurse into directories
            if metadata.is_dir() {
                Box::pin(self.search_recursive(&entry_path, pattern, results)).await?;
            }
        }

        Ok(())
    }

    /// Recursively calculate directory stats
    async fn calculate_dir_stats(&self, path: &Path) -> crate::error::Result<DirStats> {
        let mut stats = DirStats {
            total_files: 0,
            total_dirs: 0,
            total_size: 0,
        };

        self.calculate_dir_stats_recursive(path, &mut stats).await?;
        Ok(stats)
    }

    async fn calculate_dir_stats_recursive(
        &self,
        path: &Path,
        stats: &mut DirStats,
    ) -> crate::error::Result<()> {
        let mut dir = fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;

            if metadata.is_dir() {
                stats.total_dirs += 1;
                Box::pin(self.calculate_dir_stats_recursive(&entry.path(), stats)).await?;
            } else {
                stats.total_files += 1;
                stats.total_size += metadata.len();
            }
        }

        Ok(())
    }
}

/// Simple glob-like pattern matching
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern.is_empty() {
        return true;
    }

    // Simple wildcard matching
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let middle = &pattern[1..pattern.len() - 1];
        return name.contains(middle);
    }

    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return name.ends_with(suffix);
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }

    // Exact match
    name == pattern
}

#[async_trait]
impl FileSystem for LocalFileSystem {
    async fn list_dir(&self, path: &Path) -> crate::error::Result<Vec<FileEntry>> {
        let resolved = self.resolve_path(path)?;

        if !resolved.is_dir() {
            return Err(AppError::NotDirectory(path.to_path_buf()));
        }

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&resolved).await?;

        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();
            let relative_path = self.to_relative(&entry.path())?;

            entries.push(FileEntry {
                name,
                path: relative_path,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                created: metadata.created().ok(),
            });
        }

        // Sort entries: directories first, then by name
        entries.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.cmp(&b.name)
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(entries)
    }

    async fn metadata(&self, path: &Path) -> crate::error::Result<FileMetadata> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            return Err(AppError::NotFound(path.to_path_buf()));
        }

        let metadata = fs::metadata(&resolved).await?;

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(FileMetadata {
            name,
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            created: metadata.created().ok(),
            accessed: metadata.accessed().ok(),
            permissions: Some(self.get_permissions(&metadata)),
        })
    }

    async fn read_file(&self, path: &Path) -> crate::error::Result<Vec<u8>> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            return Err(AppError::NotFound(path.to_path_buf()));
        }

        if !resolved.is_file() {
            return Err(AppError::NotFile(path.to_path_buf()));
        }

        let content = fs::read(&resolved).await?;
        Ok(content)
    }

    async fn read_file_stream(
        &self,
        path: &Path,
        offset: Option<u64>,
        length: Option<u64>,
    ) -> crate::error::Result<Vec<u8>> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            return Err(AppError::NotFound(path.to_path_buf()));
        }

        if !resolved.is_file() {
            return Err(AppError::NotFile(path.to_path_buf()));
        }

        let mut file = fs::File::open(&resolved).await?;

        // Seek to offset if provided
        if let Some(offset) = offset {
            use tokio::io::AsyncSeekExt;
            file.seek(std::io::SeekFrom::Start(offset)).await?;
        }

        // Read specified length or entire file
        let mut content = Vec::new();
        if let Some(length) = length {
            let mut buffer = vec![0u8; length as usize];
            let bytes_read = file.read(&mut buffer).await?;
            buffer.truncate(bytes_read);
            content = buffer;
        } else {
            file.read_to_end(&mut content).await?;
        }

        Ok(content)
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> crate::error::Result<()> {
        let resolved = self.resolve_path(path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = resolved.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }

        fs::write(&resolved, content).await?;
        Ok(())
    }

    async fn create_dir_all(&self, path: &Path) -> crate::error::Result<()> {
        let resolved = self.resolve_path(path)?;
        fs::create_dir_all(&resolved).await?;
        Ok(())
    }

    async fn delete(&self, path: &Path) -> crate::error::Result<()> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            return Err(AppError::NotFound(path.to_path_buf()));
        }

        if resolved.is_dir() {
            fs::remove_dir_all(&resolved).await?;
        } else {
            fs::remove_file(&resolved).await?;
        }

        Ok(())
    }

    async fn rename(&self, from: &Path, to: &Path) -> crate::error::Result<()> {
        let from_resolved = self.resolve_path(from)?;
        let to_resolved = self.resolve_path(to)?;

        if !from_resolved.exists() {
            return Err(AppError::NotFound(from.to_path_buf()));
        }

        if to_resolved.exists() {
            return Err(AppError::AlreadyExists(to.to_path_buf()));
        }

        fs::rename(&from_resolved, &to_resolved).await?;
        Ok(())
    }

    async fn copy(&self, from: &Path, to: &Path) -> crate::error::Result<()> {
        let from_resolved = self.resolve_path(from)?;
        let to_resolved = self.resolve_path(to)?;

        if !from_resolved.exists() {
            return Err(AppError::NotFound(from.to_path_buf()));
        }

        if to_resolved.exists() {
            return Err(AppError::AlreadyExists(to.to_path_buf()));
        }

        if from_resolved.is_dir() {
            // Copy directory recursively
            self.copy_dir_recursive(&from_resolved, &to_resolved)
                .await?;
        } else {
            // Copy file
            fs::copy(&from_resolved, &to_resolved).await?;
        }

        Ok(())
    }

    async fn search(
        &self,
        path: &Path,
        pattern: &str,
        recursive: bool,
    ) -> crate::error::Result<Vec<SearchResult>> {
        let resolved = self.resolve_path(path)?;

        if !resolved.is_dir() {
            return Err(AppError::NotDirectory(path.to_path_buf()));
        }

        let mut results = Vec::new();

        if recursive {
            self.search_recursive(&resolved, pattern, &mut results)
                .await?;
        } else {
            let mut dir = fs::read_dir(&resolved).await?;

            while let Some(entry) = dir.next_entry().await? {
                let metadata = entry.metadata().await?;
                let name = entry.file_name().to_string_lossy().to_string();

                if matches_pattern(&name, pattern) {
                    let relative_path = self.to_relative(&entry.path())?;
                    results.push(SearchResult {
                        path: relative_path,
                        name,
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                        modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    });
                }
            }
        }

        Ok(results)
    }

    async fn dir_stats(&self, path: &Path) -> crate::error::Result<DirStats> {
        let resolved = self.resolve_path(path)?;

        if !resolved.is_dir() {
            return Err(AppError::NotDirectory(path.to_path_buf()));
        }

        self.calculate_dir_stats(&resolved).await
    }
}
