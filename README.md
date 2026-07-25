# FileDock

一个远程文件管理工具，支持 Web UI 和 WebDAV 协议访问。

## 功能特性

- 📁 **文件管理** - 浏览、上传、下载、创建、删除、重命名、复制、移动
- 🌐 **Web UI** - 现代化的浏览器界面，支持拖拽上传
- 🔗 **WebDAV** - 支持 WebDAV 协议，可映射为网络驱动器
- 🚀 **单二进制部署** - 前端资源嵌入后端，一个文件即可运行
- 🔍 **文件搜索** - 支持按文件名模式搜索
- 📊 **目录统计** - 查看目录大小和文件数量

## 技术栈

### 后端

| 技术 | 说明 |
|------|------|
| [Rust](https://www.rust-lang.org/) | 系统编程语言 |
| [Axum](https://github.com/tokio-rs/axum) | Web 框架 |
| [Tokio](https://tokio.rs/) | 异步运行时 |
| [rust-embed](https://github.com/pyros2097/rust-embed) | 静态资源嵌入 |
| [quick-xml](https://github.com/tafia/quick-xml) | XML 处理 (WebDAV) |
| [serde](https://serde.rs/) | 序列化框架 |
| [clap](https://github.com/clap-rs/clap) | 命令行参数解析 |
| [tracing](https://github.com/tokio-rs/tracing) | 日志框架 |

### 前端

| 技术 | 说明 |
|------|------|
| [React](https://react.dev/) | UI 框架 |
| [React Router](https://reactrouter.com/) | 路由管理 |
| [Tailwind CSS](https://tailwindcss.com/) | CSS 框架 |
| [shadcn/ui](https://ui.shadcn.com/) | UI 组件库 |
| [Lucide React](https://lucide.dev/) | 图标库 |
| [Vite](https://vitejs.dev/) | 构建工具 |
| [TypeScript](https://www.typescriptlang.org/) | 类型安全 |

## 快速开始

### 下载预编译版本

从 [Releases](https://github.com/your-username/FileDock/releases) 页面下载对应平台的二进制文件。

### 从源码构建

#### 前置要求

- Rust 1.70+
- Node.js 18+
- npm 或 pnpm

#### 构建步骤

```bash
# 克隆项目
git clone https://github.com/your-username/FileDock.git
cd FileDock

# 安装前端依赖
make install-frontend

# 构建完整项目
make build
```

构建完成后，可执行文件位于 `backend/target/release/filedock`。

### 运行

```bash
# 使用默认配置运行
./backend/target/release/filedock

# 指定数据目录
./backend/target/release/filedock --data-dir /path/to/data

# 指定所有参数
./backend/target/release/filedock \
  --host 0.0.0.0 \
  --port 18888 \
  --webdav-port 17777 \
  --data-dir /path/to/data
```

## 命令行参数

| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| `--host` | `-H` | `0.0.0.0` | 监听地址 |
| `--port` | `-p` | `18888` | Web UI 端口 |
| `--webdav-port` | `-w` | `17777` | WebDAV 端口 |
| `--data-dir` | `-d` | `./file_dock_data` | 数据目录路径 |

## 访问方式

### Web UI

浏览器访问 `http://localhost:18888`

功能：
- 文件浏览和管理
- 拖拽上传文件
- 文件搜索
- 目录统计

### REST API

基础地址：`http://localhost:18888/api`

主要端点：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/files?path=/` | 列出目录内容 |
| `GET` | `/api/files/read?path=/file.txt` | 读取文件 |
| `POST` | `/api/files/write?path=/file.txt` | 写入文件 |
| `POST` | `/api/files/upload?path=/` | 上传文件 |
| `GET` | `/api/files/download?path=/file.txt` | 下载文件 |
| `POST` | `/api/files` | 创建目录 |
| `DELETE` | `/api/files/delete?path=/file.txt` | 删除文件 |
| `POST` | `/api/files/rename` | 重命名/移动 |
| `POST` | `/api/files/copy` | 复制文件 |
| `GET` | `/api/files/search?path=/&pattern=*.txt` | 搜索文件 |
| `GET` | `/api/files/stats?path=/` | 目录统计 |
| `GET` | `/api/health` | 健康检查 |

### WebDAV

WebDAV 地址：`http://localhost:17777`

#### Windows

1. 打开"此电脑"
2. 点击"映射网络驱动器"
3. 输入：`http://localhost:17777`

#### macOS

1. 打开 Finder
2. 菜单栏选择"前往" -> "连接服务器"
3. 输入：`http://localhost:17777`

#### Linux

```bash
# 使用 davfs2 挂载
sudo mount -t davfs http://localhost:17777 /mnt/filedock
```

#### 常用 WebDAV 客户端

- [Cyberduck](https://cyberduck.io/)
- [WinSCP](https://winscp.net/)
- [CarotDAV](https://www.rei.to/carotdav.html)

## 项目结构

```
FileDock/
├── backend/                # Rust 后端
│   ├── src/
│   │   ├── main.rs        # 入口文件
│   │   ├── config.rs      # 配置管理
│   │   ├── error.rs       # 错误处理
│   │   ├── vfs/           # 文件系统抽象
│   │   ├── api/           # REST API
│   │   ├── protocol/      # WebDAV 协议
│   │   └── assets/        # 静态资源服务
│   └── Cargo.toml
├── frontend/               # React 前端
│   ├── app/
│   │   ├── routes/        # 页面路由
│   │   ├── components/    # UI 组件
│   │   └── lib/           # 工具函数
│   └── package.json
├── docs/                   # 文档
├── Makefile               # 构建脚本
└── README.md
```

## 开发

### 开发模式

```bash
# 终端 1：启动后端
make dev-backend

# 终端 2：启动前端
make dev-frontend
```

前端开发服务器：`http://localhost:5173`
后端 API：`http://localhost:18888`

### 代码检查

```bash
# Rust 代码检查
make lint

# 前端类型检查
cd frontend && npm run typecheck
```

### 格式化代码

```bash
make fmt
```

## 配置文件

暂不支持配置文件，所有配置通过命令行参数或环境变量设置。

## 环境变量

| 变量 | 说明 |
|------|------|
| `RUST_LOG` | 日志级别（如 `info`, `debug`, `trace`） |

示例：

```bash
RUST_LOG=debug ./backend/target/release/filedock --data-dir ./data
```

## 常见问题

### 如何修改默认端口？

```bash
./filedock --port 8080 --webdav-port 8081
```

### 如何让外网访问？

```bash
./filedock --host 0.0.0.0
```

注意：请确保防火墙已开放相应端口。

### 数据目录会自动创建吗？

是的，如果指定的数据目录不存在，会自动创建。

### 支持大文件上传吗？

支持，已配置 10GB 的请求体大小限制。

### WebDAV 连接失败？

1. 检查端口是否正确（默认 17777）
2. 检查防火墙设置
3. 尝试使用 `curl` 测试：
   ```bash
   curl -X PROPFIND http://localhost:17777/ -H "Depth: 1"
   ```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
