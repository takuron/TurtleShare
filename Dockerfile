# 运行环境
FROM debian:bookworm-slim

# 安装运行时依赖（如 TLS 证书和基础网络库）
RUN apt-get update && apt-get install -y ca-certificates sqlite3 curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 直接复制宿主机预编译好的二进制文件
COPY target/release/TurtleShare /app/TurtleShare

# 复制静态前端文件到镜像中
COPY static /app/static

# 创建数据存储目录
RUN mkdir -p /app/data

# 声明暴露的端口
EXPOSE 3000

# 启动命令，通过 -c 参数指定外部配置文件所在的路径
CMD ["/app/TurtleShare", "-c", "/app/config.toml"]
