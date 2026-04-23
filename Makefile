# 默认目标：构建核心网关与桌面端
all: build-core build-desktop

# 构建核心网关（core-gateway crate）
build-core:
	cargo build -p core-gateway

# 构建桌面端（ai-proxy-tool Tauri 工程）
build-desktop:
	cargo build -p ai-proxy-tool

# 运行桌面端（开发模式，默认走 pnpm tauri dev）
dev-desktop:
	command -v pnpm >/dev/null 2>&1 || { echo "未检测到 pnpm，请先安装：npm i -g pnpm"; exit 1; }
	cd desktop \
	&& pnpm install \
	&& (pnpm tauri dev || pnpm dlx @tauri-apps/cli@latest tauri dev)

# 一键构建并拉起桌面端（构建 core-gateway + ai-proxy-tool 并启动 UI）
up: build-core build-desktop dev-desktop

# 如需使用 cargo tauri（需安装 tauri-cli），可执行：make dev-desktop-cargo
dev-desktop-cargo:
	cd desktop && cargo tauri dev

# 仅运行网关（不通过桌面端，方便命令行调试）
run-gateway:
	cargo run -p core-gateway --bin gateway || echo "请在 core-gateway 中配置二进制入口或使用桌面端启动"

# 清理构建产物
clean:
	cargo clean

# 格式化整个工作区代码
fmt:
	cargo fmt --all

# 运行工作区所有测试
test:
	cargo test --all

.PHONY: all build-core build-desktop dev-desktop dev-desktop-cargo up run-gateway clean fmt test
