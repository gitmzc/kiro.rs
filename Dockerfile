# 多阶段构建 Dockerfile for kiro.rs
# Stage 1: 构建阶段
FROM rust:1.83-slim as builder

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

# 构建应用（这次会使用缓存的依赖）
RUN cargo build --release

# Stage 2: 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 kiro

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /build/target/release/kiro-rs /app/kiro-rs

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
