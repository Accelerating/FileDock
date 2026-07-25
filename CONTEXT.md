# FileDock 领域模型

FileDock 是一个远程文件管理服务，将本地目录通过网络协议暴露给客户端访问。

## 核心术语

**数据目录 (Data Directory)**:
用户指定的本地文件系统根目录，FileDock 管理此目录内的所有文件和子目录。
_Avoid_: 根目录、工作目录、存储目录

**协议适配器 (Protocol Adapter)**:
将文件系统操作适配为特定网络协议的组件。当前支持 WebDAV，未来可扩展 FTP 等。
_Avoid_: 协议处理器、协议模块

**文件系统抽象 (File System Abstraction)**:
对底层文件系统操作的统一接口，所有协议和 API 都通过此层操作文件。
_Avoid_: VFS、虚拟文件系统

**文件条目 (File Entry)**:
数据目录中的一个文件或目录，包含名称、路径、类型、大小、修改时间等元数据。
_Avoid_: 文件对象、文件信息

**文件元数据 (File Metadata)**:
文件的详细属性信息，包括名称、路径、大小、创建时间、修改时间、访问时间、权限等。
_Avoid_: 文件属性、文件信息

**目录统计 (Directory Statistics)**:
目录的统计信息，包括文件数量、子目录数量、总大小等。
_Avoid_: 目录信息、目录属性

**搜索结果 (Search Result)**:
文件搜索的结果，包含匹配的文件路径、名称、类型、大小等信息。
_Avoid_: 搜索条目、匹配结果

**Web 管理界面 (Web Admin UI)**:
嵌入到后端二进制中的浏览器界面，为没有专用客户端的用户提供文件管理能力。
_Avoid_: 前端、Web UI、管理后台

## API 响应

**标准化响应 (API Response)**:
所有 API 端点都使用统一的响应格式，包含 success、data、error 字段。
_Avoid_: 响应格式、返回格式

**分页响应 (Paginated Response)**:
列表类 API 支持分页，返回 items、total、page、page_size、has_more 字段。
_Avoid_: 分页格式、列表格式

**批量操作 (Batch Operation)**:
支持一次请求操作多个文件，返回 success_count、failure_count、failures 字段。
_Avoid_: 批量处理、多文件操作

## 前端组件

**文件浏览器 (File Browser)**:
主界面组件，显示当前目录的文件列表，支持导航、排序、选择等操作。
_Avoid_: 文件管理器、文件列表页

**面包屑导航 (Breadcrumb Navigation)**:
显示当前路径的层级导航，支持点击跳转到上级目录。
_Avoid_: 路径导航、目录导航

**文件列表 (File List)**:
显示目录内容的组件，支持排序、选择、右键菜单等操作。
_Avoid_: 文件表格、目录列表

**上传对话框 (Upload Dialog)**:
文件上传组件，支持拖放和多文件上传。
_Avoid_: 文件上传框、上传界面

**创建对话框 (Create Dialog)**:
创建新文件或目录的组件，支持选择类型和输入内容。
_Avoid_: 新建对话框、创建界面

## WebDAV 协议

**WebDAV (Web Distributed Authoring and Versioning)**:
HTTP 的扩展协议，允许用户协作编辑和管理远程 Web 服务器上的文件。
_Avoid_: DAV、WebDAV 协议

**PROPFIND**:
WebDAV 方法，用于获取资源的属性信息。
_Avoid_: 属性查询、属性查找

**MKCOL**:
WebDAV 方法，用于创建集合（目录）。
_Avoid_: 创建目录、创建集合

**DAV 属性**:
WebDAV 资源的属性，如创建日期、修改日期、内容类型等。
_Avoid_: 文件属性、资源属性

## 部署

**单二进制部署 (Single Binary Deployment)**:
将前端资源嵌入到后端二进制文件中，只需分发一个可执行文件即可运行完整应用。
_Avoid_: 单文件部署、一体化部署

**数据目录 (Data Directory)**:
用户指定的本地文件系统目录，FileDock 管理此目录内的所有文件和子目录。
_Avoid_: 存储目录、工作目录

**构建脚本 (Build Script)**:
自动化构建流程的脚本，包括前端构建、资源复制、后端编译等步骤。
_Avoid_: 构建工具、构建配置

## 配置

**命令行参数 (Command Line Arguments)**:
通过命令行传递给程序的参数，用于配置程序行为。
_Avoid_: 启动参数、程序参数

**Web 端口 (Web Port)**:
Web UI 服务监听的端口号，默认 18888。
_Avoid_: 前端端口、UI 端口

**WebDAV 端口 (WebDAV Port)**:
WebDAV 服务监听的端口号，默认 17777。
_Avoid_: DAV 端口、协议端口

**监听地址 (Listen Address)**:
服务器监听的网络地址，默认 0.0.0.0（所有地址）。
_Avoid_: 绑定地址、主机地址

## 文件操作

**创建 (Create)**:
在数据目录中创建新文件或目录。
_Avoid_: 新建、添加

**读取 (Read)**:
获取文件内容或列出目录内容。支持流式读取和范围读取。
_Avoid_: 查询、获取、下载

**更新 (Update)**:
修改文件内容、重命名或移动文件/目录。
_Avoid_: 编辑、修改

**删除 (Delete)**:
从数据目录中移除文件或目录。
_Avoid_: 移除、销毁

**上传 (Upload)**:
通过 HTTP multipart 表单上传文件到数据目录。
_Avoid_: 导入、添加文件

**下载 (Download)**:
从数据目录下载文件，包含正确的 Content-Type 和 Content-Disposition 头。
_Avoid_: 导出、获取文件

**搜索 (Search)**:
根据文件名模式在目录中搜索文件，支持递归搜索和通配符匹配。
_Avoid_: 查找、查询

## 架构组件

**后端服务 (Backend Service)**:
Rust 实现的服务端，处理文件操作、协议服务和 API 请求。
_Avoid_: 服务端、服务器

**静态资源 (Static Assets)**:
前端构建产物（HTML、CSS、JS），嵌入到后端二进制文件中。
_Avoid_: 前端资源、静态文件
