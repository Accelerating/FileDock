# FileDock 技术设计

## 项目结构

```
FileDock/
├── backend/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs              # 入口，解析配置，启动服务
│   │   ├── config.rs            # 配置管理
│   │   ├── vfs/                 # 文件系统抽象层
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs        # FileSystem trait
│   │   │   └── local.rs         # 本地文件系统实现
│   │   ├── protocol/            # 协议适配器
│   │   │   ├── mod.rs
│   │   │   └── webdav/
│   │   │       ├── mod.rs
│   │   │       └── handler.rs
│   │   ├── api/                 # REST API
│   │   │   ├── mod.rs
│   │   │   └── files.rs
│   │   ├── assets/              # 嵌入的前端资源
│   │   │   └── mod.rs
│   │   └── error.rs             # 错误处理
│   └── assets/                  # 前端构建产物（构建时复制）
├── frontend/
│   ├── app/
│   │   ├── root.tsx
│   │   ├── routes.ts
│   │   ├── routes/
│   │   │   ├── home.tsx
│   │   │   └── browse.$.tsx     # 文件浏览器路由
│   │   ├── components/
│   │   │   ├── FileList.tsx
│   │   │   ├── FileItem.tsx
│   │   │   ├── Breadcrumb.tsx
│   │   │   ├── UploadDialog.tsx
│   │   │   └── CreateDialog.tsx
│   │   └── lib/
│   │       └── api.ts           # API 客户端
│   ├── package.json
│   ├── vite.config.ts
│   └── tailwind.config.ts
├── docs/
│   └── adr/
├── CONTEXT.md
└── Makefile                     # 构建脚本
```

## 依赖选择

### 后端 (Cargo.toml)

```toml
[package]
name = "filedock"
version = "0.1.0"
edition = "2024"

[dependencies]
# Web 框架
axum = { version = "0.7", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 异步 trait
async-trait = "0.1"

# 文件系统操作
tokio-util = { version = "0.7", features = ["io"] }

# 静态资源嵌入
rust-embed = "8"

# 错误处理
thiserror = "1"
anyhow = "1"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 配置
clap = { version = "4", features = ["derive", "env"] }

# MIME 类型
mime_guess = "2"
```

### 前端 (package.json)

```json
{
  "dependencies": {
    "@react-router/node": "^8",
    "@react-router/serve": "^8",
    "react": "^19",
    "react-dom": "^19",
    "react-router": "^8"
  },
  "devDependencies": {
    "@react-router/dev": "^8",
    "@tailwindcss/vite": "^4",
    "tailwindcss": "^4",
    "typescript": "^5",
    "vite": "^8"
  }
}
```

## 实现阶段

### 阶段 1: 基础框架 ✅
1. 设置 Rust 项目结构
2. 实现配置解析（命令行参数）
3. 实现基本的 Axum 服务器
4. 设置前端项目

### 阶段 2: 文件系统抽象 ✅
1. 定义 FileSystem trait
2. 实现本地文件系统适配器
3. 添加路径安全检查（防止目录遍历攻击）
4. 实现文件搜索功能
5. 实现目录统计功能
6. 支持文件流式读取（大文件支持）

### 阶段 3: REST API ✅
1. 实现文件列表 API（支持分页和排序）
2. 实现文件上传/下载 API
3. 实现文件操作 API（创建、删除、重命名、移动、复制）
4. 实现文件搜索 API（支持分页）
5. 实现目录统计 API
6. 实现批量操作 API（批量删除、批量复制）
7. 实现健康检查 API
8. 标准化响应格式（统一的 ApiResponse 包装）

### 阶段 4: Web 前端 ✅
1. 初始化 shadcn/ui 项目
2. 添加常用组件（Button, Card, Dialog, Table, Input, DropdownMenu 等）
3. 实现文件浏览器界面
4. 实现面包屑导航
5. 实现文件列表组件（支持排序和选择）
6. 实现文件上传对话框（支持拖放）
7. 实现创建文件/目录对话框
8. 实现重命名对话框
9. 实现删除确认对话框
10. 实现 API 客户端库
11. 实现工具函数（文件大小格式化、日期格式化等）

### 阶段 5: WebDAV 协议 ✅
1. 实现 WebDAV 协议处理器
2. 支持 WebDAV 方法（OPTIONS, GET, HEAD, PUT, DELETE, MKCOL, COPY, MOVE, PROPFIND, PROPPATCH, LOCK, UNLOCK）
3. 实现 XML 序列化/反序列化
4. 支持目录列表和文件属性查询
5. 支持文件创建、删除、复制、移动
6. 支持锁机制（基本实现）

### 阶段 6: 打包与部署 ✅
1. 实现前端资源嵌入
2. 创建构建脚本（Makefile）
3. 测试单二进制部署
4. 验证完整功能（API、前端、WebDAV）
5. 修复大文件上传问题（统一使用 bytes() 读取文件内容）
6. 修复目录导航问题（路径处理）
7. 修复文件删除问题（返回JSON响应，支持非空目录二次确认）

## API 设计详情

### 文件列表

```
GET /api/files?path=/some/dir

Response 200:
{
  "path": "/some/dir",
  "entries": [
    {
      "name": "file.txt",
      "path": "file.txt",
      "is_dir": false,
      "size": 1024,
      "modified": "2024-01-01T00:00:00Z",
      "created": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### 文件元数据

```
GET /api/files/metadata?path=/some/dir/file.txt

Response 200:
{
  "path": "/some/dir/file.txt",
  "name": "file.txt",
  "is_dir": false,
  "size": 1024,
  "modified": "2024-01-01T00:00:00Z",
  "created": "2024-01-01T00:00:00Z",
  "accessed": "2024-01-01T00:00:00Z",
  "permissions": {
    "readable": true,
    "writable": true,
    "executable": false
  }
}
```

### 读取文件

```
GET /api/files/read?path=/some/dir/file.txt
GET /api/files/read?path=/some/dir/file.txt&offset=0&length=1024

Response 200:
Content-Type: application/octet-stream

<binary data>
```

### 写入文件

```
POST /api/files/write?path=/some/dir/file.txt
Content-Type: text/plain

<file content>

Response 200
```

### 上传文件

```
POST /api/files/upload?path=/some/dir
Content-Type: multipart/form-data

file: <binary data>

Response 201
```

### 下载文件

```
GET /api/files/download?path=/some/dir/file.txt

Response 200:
Content-Type: <mime type>
Content-Disposition: attachment; filename="file.txt"
Content-Length: 1024

<binary data>
```

### 创建目录

```
POST /api/files
Content-Type: application/json

{
  "path": "/some/dir/newdir"
}

Response 201
```

### 删除文件/目录

```
DELETE /api/files/delete?path=/some/dir/file.txt

Response 200
```

### 重命名/移动

```
POST /api/files/rename
Content-Type: application/json

{
  "from": "/some/dir/file.txt",
  "to": "/some/dir/newfile.txt"
}

Response 200
```

### 复制

```
POST /api/files/copy
Content-Type: application/json

{
  "from": "/some/dir/file.txt",
  "to": "/other/dir/file.txt"
}

Response 201
```

### 搜索文件

```
GET /api/files/search?path=/some/dir&pattern=*.txt&recursive=true

Response 200:
{
  "pattern": "*.txt",
  "results": [
    {
      "path": "file.txt",
      "name": "file.txt",
      "is_dir": false,
      "size": 1024,
      "modified": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### 目录统计

```
GET /api/files/stats?path=/some/dir

Response 200:
{
  "success": true,
  "data": {
    "path": "/some/dir",
    "stats": {
      "total_files": 10,
      "total_dirs": 3,
      "total_size": 102400
    }
  }
}
```

### 健康检查

```
GET /api/health

Response 200:
{
  "success": true,
  "data": {
    "status": "ok",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "data_dir": "/path/to/data"
  }
}
```

### 批量删除

```
POST /api/files/batch/delete
Content-Type: application/json

{
  "paths": [
    "/some/dir/file1.txt",
    "/some/dir/file2.txt"
  ]
}

Response 200:
{
  "success": true,
  "data": {
    "success_count": 2,
    "failure_count": 0,
    "failures": []
  }
}
```

### 批量复制

```
POST /api/files/batch/copy
Content-Type: application/json

{
  "operations": [
    {
      "from": "/some/dir/file1.txt",
      "to": "/other/dir/file1.txt"
    },
    {
      "from": "/some/dir/file2.txt",
      "to": "/other/dir/file2.txt"
    }
  ]
}

Response 200:
{
  "success": true,
  "data": {
    "success_count": 2,
    "failure_count": 0,
    "failures": []
  }
}
```

### 分页和排序

所有列表端点都支持分页和排序参数：

```
GET /api/files?path=/some/dir&page=0&page_size=10&sort_by=name&sort_order=asc
GET /api/files/search?path=/some/dir&pattern=*.txt&page=0&page_size=20
```

参数说明：
- `page`: 页码（从 0 开始）
- `page_size`: 每页数量（默认 100）
- `sort_by`: 排序字段（name, size, modified, created）
- `sort_order`: 排序顺序（asc, desc）

## 安全考虑

1. **路径遍历防护**: 所有路径操作必须验证不超出数据目录
2. **文件名验证**: 拒绝非法文件名（如包含 `..`、`/` 等）
3. **大小限制**: 可配置的文件上传大小限制
4. **并发控制**: 文件操作的并发安全

## 配置方式

### 命令行参数
```bash
# 使用默认配置（Web端口18888，WebDAV端口17777，数据目录./file_dock_data）
filedock

# 指定数据目录
filedock --data-dir /path/to/data

# 指定主机和端口
filedock --host 0.0.0.0 --port 8080 --webdav-port 8081

# 完整配置
filedock --host 0.0.0.0 --port 18888 --webdav-port 17777 --data-dir /path/to/data
```

### 参数说明
| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| --host | -H | 0.0.0.0 | 服务器监听地址 |
| --port | -p | 18888 | Web UI 端口 |
| --webdav-port | -w | 17777 | WebDAV 端口 |
| --data-dir | -d | ./file_dock_data | 数据目录路径 |
