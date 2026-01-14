# 多阶段构建 Dockerfile for kiro.rs
# Stage 1: 前端构建阶段
FROM node:20-slim as frontend-builder

WORKDIR /frontend

# 复制前端配置文件
COPY web-admin/package*.json ./

# 安装依赖
RUN npm ci

# 复制前端源代码
COPY web-admin/ ./

# 构建前端
RUN npm run build

# Stage 2: 后端构建阶段
FROM rust:1.83-slim as backend-builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /build

# 复制 Cargo 配置文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 从前端构建阶段复制构建产物到 web-admin/dist
# 这样 rust-embed 可以在编译时嵌入前端文件
COPY --from=frontend-builder /frontend/dist ./web-admin/dist

# 删除之前空 main 的编译产物，强制重新编译真正的代码
RUN rm -rf target/release/kiro-rs target/release/deps/kiro* target/release/.fingerprint/kiro*

# 构建应用（这次会使用缓存的依赖，并嵌入前端文件）
RUN cargo build --release

# Stage 3: 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 kiro

# 设置工作目录
WORKDIR /app

# 从后端构建阶段复制二进制文件（已包含嵌入的前端文件）
COPY --from=backend-builder /build/target/release/kiro-rs /app/kiro-rs

# 创建必要的目录
RUN mkdir -p /app/data /app/logs && \
    chown -R kiro:kiro /app

# 切换到非 root 用户
USER kiro

# 暴露端口
EXPOSE 8990

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "wget --no-verbose --tries=1 --spider http://localhost:8990/v1/models || exit 1"]

# 启动命令
ENTRYPOINT ["/app/kiro-rs"]
CMD ["-c", "/app/config.json", "--credentials", "/app/credentials.json"]
