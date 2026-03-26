mod config;
mod utils;
mod error;

use crate::config::Config;
use crate::utils::hash;
use axum::{routing::get, Json, Router};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::Serialize;

/// TurtleShare: A membership content distribution platform.
//
// // TurtleShare：一个会员内容分发平台。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the configuration file.
    //
    // // 配置文件路径。
    #[arg(short = 'c', long = "config", default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate an Argon2id hash for a password.
    //
    // // 为密码生成 Argon2id 哈希。
    HashPw {
        /// The raw password to hash.
        //
        // // 要哈希的原始密码。
        password: String,
    },
}

/// Standard API response wrapper.
//
// // 标准 API 响应包装。
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志。
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. 解析命令行参数。
    let args = Args::parse();

    // 3. 处理子命令（如果提供）。
    if let Some(Commands::HashPw { password }) = args.command {
        let hash = hash::hash_password(&password)?;
        println!("Raw password: {}", password);
        println!("Argon2id Hash (PHC format):");
        println!("{}", hash);
        return Ok(());
    }

    tracing::info!("Loading configuration from: {}", args.config);

    // 4. 加载配置文件。
    let config = Config::load(&args.config)?;
    tracing::info!("Successfully loaded configuration for: {}", config.site_info.name);

    // 5. 定义路由。
    let app = Router::new()
        .route("/", get(|| async { "TurtleShare API is running!" }))
        .route("/api/health", get(health_check))
        .route("/api/public/site-info", get({
            let site_info = config.site_info.clone();
            move || async { Json(ApiResponse { success: true, data: site_info }) }
        }));

    // 6. 启动服务器。
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse::<SocketAddr>()?;

    tracing::info!("Server starting at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Simple health check endpoint.
//
// // 简单的健康检查端点。
async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "status": "ok" }),
    })
}
