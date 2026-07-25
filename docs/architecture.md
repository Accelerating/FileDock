# FileDock 架构设计

## 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    FileDock 二进制文件                        │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                  Rust 后端服务                        │  │
│  │                                                      │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │  WebDAV     │  │   REST      │  │   静态资源   │  │  │
│  │  │  协议处理   │  │   API       │  │   服务      │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │  │
│  │         │                │                │          │  │
│  │         └────────────────┼────────────────┘          │  │
│  │                          │                           │  │
│  │                 ┌────────┴────────┐                  │  │
│  │                 │   文件系统抽象   │                  │  │
│  │                 └────────┬────────┘                  │  │
│  │                          │                           │  │
│  │                 ┌────────┴────────┐                  │  │
│  │                 │   数据目录      │                  │  │
│  │                 └─────────────────┘                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              嵌入的前端静态资源                        │  │
│  │         (React Router 构建产物)                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 请求路由

```
HTTP 请求
    │
    ├─ /webdav/* ──→ WebDAV 协议处理器 ──→ 文件系统抽象
    │
    ├─ /api/* ──→ REST API 处理器 ──→ 文件系统抽象
    │
    └─ /* ──→ 静态资源服务 (嵌入的前端)
```

## 核心模块设计

### 1. 文件系统抽象层 (vfs)

```rust
// 核心 trait
trait FileSystem {
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    async fn write_file(&self, path: &Path, content: &[u8]) -> Result<()>;
    async fn create_dir(&self, path: &Path) -> Result<()>;
    async fn delete(&self, path: &Path) -> Result<()>;
    async fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    async fn copy(&self, from: &Path, to: &Path) -> Result<()>;
    async fn list_dir(&self, path: &Path) -> Result<Vec<FileEntry>>;
    async fn metadata(&self, path: &Path) -> Result<FileMetadata>;
    async fn exists(&self, path: &Path) -> Result<bool>;
}

// 文件条目
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: SystemTime,
}

// 文件元数据
struct FileMetadata {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    created: Option<SystemTime>,
    modified: SystemTime,
    accessed: Option<SystemTime>,
}
```

### 2. 协议适配器层

```rust
// 协议适配器 trait
trait ProtocolAdapter {
    fn name(&self) -> &str;
    fn routes(&self) -> Router;
}

// WebDAV 适配器
struct WebDavAdapter {
    fs: Arc<dyn FileSystem>,
}

// 未来可扩展
// struct FtpAdapter { ... }
// struct SmbAdapter { ... }
```

### 3. REST API 设计

```
GET    /api/files?path=          # 列出目录内容
GET    /api/files/*path          # 下载文件
POST   /api/files/*path          # 创建文件/目录
PUT    /api/files/*path          # 更新文件内容
DELETE /api/files/*path          # 删除文件/目录
PATCH  /api/files/*path          # 重命名/移动
POST   /api/files/*path/copy     # 复制文件/目录
```

### 4. 前端路由设计

```
/                           # 首页，重定向到文件浏览器
/browse/*                   # 文件浏览器
/browse/                    # 根目录浏览
/browse/subfolder/          # 子目录浏览
```

## 技术选型

### 后端
- **Web 框架**: Axum (Rust 异步 web 框架)
- **WebDAV**: 自实现或使用 `dav-server` crate
- **静态资源嵌入**: `rust-embed` 或 `include_bytes!`
- **文件系统操作**: `tokio::fs` (异步文件操作)
- **序列化**: `serde` + `serde_json`

### 前端
- **框架**: React Router v8
- **样式**: Tailwind CSS v4
- **构建**: Vite
- **类型**: TypeScript

## 构建流程

```bash
# 1. 构建前端
cd frontend && npm run build

# 2. 将前端构建产物复制到后端资源目录
cp -r frontend/build/* backend/assets/

# 3. 构建后端（前端资源嵌入二进制）
cd backend && cargo build --release

# 最终产物: backend/target/release/filedock
```

## 配置方式

### 命令行参数
```bash
filedock --data-dir /path/to/data --port 8080 --host 0.0.0.0
```

### 环境变量
```bash
FILEDOCK_DATA_DIR=/path/to/data
FILEDOCK_PORT=8080
FILEDOCK_HOST=0.0.0.0
```

### 配置文件 (可选)
```toml
# filedock.toml
[data]
dir = "/path/to/data"

[server]
host = "0.0.0.0"
port = 8080

[webdav]
enabled = true
prefix = "/webdav"
```
