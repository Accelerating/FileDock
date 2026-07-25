# FileDock 构建脚本

.PHONY: all build-frontend build-backend dev clean run test-api install-frontend

# 默认目标：构建完整的单二进制
all: build

# 构建前端
build-frontend:
	cd frontend && npm run build

# 复制前端资源到后端
copy-assets: build-frontend
	rm -rf backend/assets
	cp -r frontend/build/client backend/assets

# 构建后端（包含嵌入的前端资源）
build-backend:
	cd backend && cargo build --release

# 完整构建
build: copy-assets build-backend
	@echo "构建完成！"
	@echo "可执行文件: backend/target/release/filedock"

# 开发模式：启动后端
dev-backend:
	cd backend && cargo run

# 开发模式：启动前端
dev-frontend:
	cd frontend && npm run dev

# 运行（使用默认配置）
run:
	cd backend && cargo run

# 清理构建产物
clean:
	cd frontend && rm -rf build node_modules
	cd backend && cargo clean
	rm -rf backend/assets

# 安装前端依赖
install-frontend:
	cd frontend && npm install
	cd frontend && npx shadcn@latest init --defaults

# 安装所有依赖
install: install-frontend

# 运行测试
test:
	cd backend && cargo test
	cd frontend && npm run typecheck

# 代码格式化
fmt:
	cd backend && cargo fmt

# 代码检查
lint:
	cd backend && cargo clippy

# 测试 API
test-api:
	cd backend && cargo run -- --data-dir /tmp/filedock-test &
	sleep 5
	@echo "=== 测试健康检查 ==="
	curl -s http://localhost:18888/api/health
	@echo ""
	@echo "=== 测试分页目录列表 ==="
	curl -s "http://localhost:18888/api/files?path=/&page=0&page_size=10"
	@echo ""
	@echo "=== 测试文件写入 ==="
	curl -s -X POST "http://localhost:18888/api/files/write?path=/test.txt" -d "Hello, FileDock!"
	@echo ""
	@echo "=== 测试文件读取 ==="
	curl -s "http://localhost:18888/api/files/read?path=/test.txt"
	@echo ""
	@echo "=== 测试 WebDAV ==="
	curl -s -X PROPFIND http://localhost:17777/ -H "Depth: 1" | head -50
	@echo ""
	@echo "=== 测试 WebDAV MKCOL ==="
	curl -s -X MKCOL http://localhost:17777/newdir -D-
	@echo ""
	@echo "=== 测试 WebDAV PUT ==="
	curl -s -X PUT http://localhost:17777/newfile.txt -d "Hello WebDAV" -D-
	@echo ""
	@echo "=== 测试 WebDAV GET ==="
	curl -s http://localhost:17777/newfile.txt
	@echo ""
	pkill -f "target/debug/filedock" || true
	rm -rf /tmp/filedock-test
