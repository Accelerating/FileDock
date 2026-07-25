pub mod traits;
pub mod local;

pub use traits::FileSystem;
pub use traits::{FileEntry, FilePermissions, DirStats, SearchResult};
pub use local::LocalFileSystem;
