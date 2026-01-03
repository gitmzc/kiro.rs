# Docker 部署指南

本文档介绍如何使用 Docker 和 Docker Compose 将 kiro.rs 部署到 NAS 或其他服务器上。

## 前置要求

- Docker 20.10 或更高版本
- Docker Compose 2.0 或更高版本
- 至少 1GB 可用磁盘空间

## 快速开始

### 1. 准备配置文件

在项目根目录创建 `config.json` 配置文件：

```json
{
  "host": "0.0.0.0",
  "port": 8990,
  "apiKey": "sk-kiro-rs-your-custom-api-key",
  "region": "us-east-1"
}
```

**重要**: 将 `host` 设置为 `0.0.0.0` 以允许容器外部访问。

### 2. 准备凭据文件

创建 `credentials.json` 文件（支持单凭据或多凭据格式）：

**单凭据示例**:
```json
{
  "refreshToken": "your-refresh-token",
  "expiresAt": "2025-12-31T02:32:45.144Z",
  "authMethod": "social"
}
```

**多凭据示例**:
```json
[
  {
    "refreshToken": "first-refresh-token",
    "expiresAt": "2025-12-31T02:32:45.144Z",
    "authMethod": "social",
    "priority": 0
  },
  {
    "refreshToken": "second-refresh-token",
    "expiresAt": "2025-12-31T02:32:45.144Z",
    "authMethod": "idc",
    "clientId": "your-client-id",
    "clientSecret": "your-client-secret",
    "priority": 1
  }
]
```

### 3. 构建并启动服务

```bash
# 构建镜像并启动容器
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

## 目录结构

部署后的目录结构如下：

```
kiro.rs/
├── docker-compose.yml    # Docker Compose 配置
├── Dockerfile            # Docker 镜像构建文件
├── config.json          # 服务配置（必需）
├── credentials.json     # 凭据配置（必需）
├── data/                # 数据持久化目录（自动创建）
└── logs/                # 日志目录（自动创建）
```

## 配置说明

### docker-compose.yml 配置项

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `ports` | 端口映射 | `8990:8990` |
| `RUST_LOG` | 日志级别 | `info` |
| `TZ` | 时区设置 | `Asia/Shanghai` |

### 修改端口

如果需要修改服务端口，编辑 `docker-compose.yml`：

```yaml
ports:
  - "9000:8990"  # 将主机的 9000 端口映射到容器的 8990 端口
```

同时需要在 `config.json` 中保持 `port: 8990` 不变（容器内部端口）。

### 日志级别

可以通过环境变量调整日志级别：

```yaml
environment:
  - RUST_LOG=debug  # 可选: trace, debug, info, warn, error
```

## NAS 部署指南

### 群晖 NAS (Synology)

1. 安装 Docker 套件（通过套件中心）
2. 使用 SSH 登录 NAS
3. 上传项目文件到 NAS（如 `/volume1/docker/kiro-rs/`）
4. 进入项目目录并执行：

```bash
cd /volume1/docker/kiro-rs
docker-compose up -d
```

5. 在 Docker 套件中可以查看容器状态和日志

### 威联通 NAS (QNAP)

1. 安装 Container Station
2. 使用 SSH 登录 NAS
3. 上传项目文件到 NAS
4. 执行部署命令：

```bash
cd /share/Container/kiro-rs
docker-compose up -d
```

### 其他 NAS 系统

只要支持 Docker 和 Docker Compose，部署步骤基本相同：
1. 确保 Docker 服务已启动
2. 上传项目文件
3. 执行 `docker-compose up -d`

## 健康检查

容器内置健康检查，每 30 秒检查一次服务状态：

```bash
# 查看容器健康状态
docker ps

# 查看详细健康检查日志
docker inspect kiro-rs | grep -A 10 Health
```

## 数据持久化

以下目录会自动挂载到主机，确保数据持久化：

- `./data` - 应用数据
- `./logs` - 日志文件
- `./config.json` - 配置文件（只读）
- `./credentials.json` - 凭据文件（只读）

## 更新服务

### 更新代码

```bash
# 拉取最新代码
git pull

# 重新构建并启动
docker-compose up -d --build
```

### 更新配置

修改 `config.json` 或 `credentials.json` 后：

```bash
# 重启容器以应用新配置
docker-compose restart
```

## 故障排查

### 查看日志

```bash
# 查看实时日志
docker-compose logs -f

# 查看最近 100 行日志
docker-compose logs --tail=100

# 查看特定时间的日志
docker-compose logs --since 30m
```

### 容器无法启动

1. 检查配置文件是否正确：
```bash
cat config.json
cat credentials.json
```

2. 检查端口是否被占用：
```bash
netstat -tuln | grep 8990
```

3. 查看详细错误信息：
```bash
docker-compose logs
```

### 无法访问服务

1. 确认容器正在运行：
```bash
docker ps | grep kiro-rs
```

2. 检查防火墙设置（NAS 可能需要在控制面板中开放端口）

3. 测试服务是否响应：
```bash
curl http://localhost:8990/v1/models
```

## 安全建议

1. **保护凭据文件**: 确保 `credentials.json` 权限设置为 600
```bash
chmod 600 credentials.json
```

2. **使用强 API Key**: 在 `config.json` 中设置复杂的 `apiKey`

3. **限制网络访问**: 如果只需要内网访问，可以修改端口映射：
```yaml
ports:
  - "127.0.0.1:8990:8990"  # 只允许本地访问
```

4. **定期更新**: 定期拉取最新代码并重新构建镜像

## 性能优化

### 资源限制

如果需要限制容器资源使用，可以在 `docker-compose.yml` 中添加：

```yaml
services:
  kiro-rs:
    # ... 其他配置 ...
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
```

### 日志轮转

为避免日志文件过大，可以配置 Docker 日志驱动：

```yaml
services:
  kiro-rs:
    # ... 其他配置 ...
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

## 使用示例

服务启动后，可以通过以下方式使用：

```bash
# 测试连接
curl http://your-nas-ip:8990/v1/models

# 发送消息
curl http://your-nas-ip:8990/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-kiro-rs-your-custom-api-key" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "max_tokens": 1024,
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

## 备份与恢复

### 备份

```bash
# 备份配置和数据
tar -czf kiro-rs-backup-$(date +%Y%m%d).tar.gz \
  config.json \
  credentials.json \
  data/ \
  logs/
```

### 恢复

```bash
# 解压备份
tar -xzf kiro-rs-backup-20260103.tar.gz

# 重启服务
docker-compose restart
```

## 支持

如遇到问题，请查看：
- [项目 README](README.md)
- [GitHub Issues](https://github.com/gitmzc/kiro.rs/issues)
