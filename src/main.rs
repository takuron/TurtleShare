mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod utils;

use crate::config::Config;
use crate::handlers::create_router;
use crate::utils::{hash, hashid::HashIdManager, jwt::JwtManager};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    /// Require an existing database file. If not present, the server will error out instead of creating it.
    //
    // // 强制要求存在现有的数据库文件。如果不存在，服务器将报错而不是创建它。
    #[arg(long = "require-existing-db")]
    require_existing_db: bool,
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
    tracing::info!(
        "Successfully loaded configuration for: {}",
        config
            .siteinfo
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("TurtleShare")
    );

    // 5.1. 初始化数据库连接。
    let pool = db::init_db(&config.database.path, args.require_existing_db).await?;
    tracing::info!("Database initialized successfully.");

    // 5.2. 初始化JWT管理器。
    let jwt_manager = Arc::new(
        JwtManager::new(
            pool.clone(),
            config.jwt.base_secret.clone(),
            config.jwt.expiry_hours,
            config.jwt.rotation_days,
        )
        .await?,
    );
    tracing::info!("JWT manager initialized successfully.");

    // 5.3. 初始化HashID管理器。
    let hashid_manager = Arc::new(HashIdManager::new(
        &config.jwt.base_secret,
        config.hashid.min_length,
    )?);
    tracing::info!("HashID manager initialized successfully.");

    // 5.4. 启动JWT密钥轮换后台任务。
    {
        let jwt_manager = jwt_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // 每天检查一次
            loop {
                interval.tick().await;
                if let Err(e) = jwt_manager.check_and_rotate().await {
                    tracing::error!("JWT rotation check failed: {}", e);
                }
            }
        });
    }

    // 5.5. 确保存储目录存在。
    if !std::path::Path::new(&config.storage.files_path).exists() {
        std::fs::create_dir_all(&config.storage.files_path)?;
        tracing::info!(
            "Created storage directory at: {}",
            config.storage.files_path
        );
    }

    // 6. 定义路由。
    let app = create_router(
        config.clone(),
        jwt_manager.clone(),
        hashid_manager.clone(),
        pool.clone(),
    )?
    .into_make_service_with_connect_info::<SocketAddr>();

    // 7. 启动服务器。
    let addr = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;

    tracing::info!("Server starting at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
