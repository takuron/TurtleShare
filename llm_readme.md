# TurtleShare Technical Overview for LLM

!!! IMPORTANT: YOU MUST FOLLOW THE CODING SPECIFICATIONS IN SECTION 1 FOR EVERY RESPONSE. !!!

## 1. LLM Coding Specification

To ensure consistency and maintainability when using an LLM for development, the following rules must be strictly followed.

### 1.1. Documentation Synchronicity
*   **Rule:** Any change that modifies the project's structure, adds or removes a module, or alters the core public API **must** be accompanied by a corresponding update to this document (`llm_readme.md`).
*   **Goal:** This document must always serve as a reliable and up-to-date source of truth for the project's architecture.

### 1.2. Changelog Maintenance
*   **Rule:** After every coding task (e.g., adding a feature, fixing a bug), a concise summary of the changes must be logged by executing the `llm_log.py` script.
*   **Format:** The command should be `python llm_log.py "Your concise log message."`. The script will automatically handle timestamping and appending to `llm_log.txt`.
*   **Goal:** To maintain a persistent, append-only log of all modifications made by the LLM, with a standardized format.

### 1.3. Internal Code Commenting
*   **Rule:** Internal implementation logic should be commented in Chinese, focusing on the sequence and purpose of operations.
*   **Style:** Comments should be brief and precede the code block they describe.
*   **Example:**
    ```rust
    // 1. 首先，确保文件存在于数据库中。这是一个快速检查。
    match self.find_by_hash(hash)? {
        QueryResult::NotFound => return Err(UpdateError::FileNotFound(hash.to_string())),
        QueryResult::Found(_) => {
            // 文件存在，继续进行哈希验证。
        }
    }

    // 2. 对存储的数据执行实际的哈希计算。这是 I/O 密集型操作。
    verify_encrypted_file_hash(self.storage.as_ref(), hash)
    ```

### 1.4. Public API Documentation
*   **Rule:** All public APIs (structs, functions, and fields) must be documented using a dual-language (English and Chinese) format.
*   **Format:**
    1.  **English:** Use standard Rustdoc (`///`) comments. The comment should include a summary, a detailed description, and sections for `# Arguments`, `# Returns`, and `# Errors` where applicable.
    2.  **Chinese:** Immediately following the English comment, provide a direct translation, with each line prefixed by `// //`.
*   **Example:**
    ```rust
    /// Creates a new Vault at the specified path.
    ///
    /// This will create the root directory and initialize the `master.json`
    /// configuration and the `master.db` database.
    ///
    /// # Arguments
    /// * `root_path` - The path where the vault metadata will be stored.
    ///
    /// # Errors
    /// Returns `CreateError` if the directory already exists and is not empty.
    //
    // // 在指定路径创建一个新的保险库。
    // //
    // // 这将创建根目录并初始化 `master.json` 配置文件和 `master.db` 数据库。
    // //
    // // # 参数
    // // * `root_path` - 将存储保险库元数据的路径。
    // //
    // // # 错误
    // // 如果目录已存在且不为空，则返回 `CreateError`。
    pub fn create_vault(...) -> Result<Vault, CreateError> { ... }
    ```

### 1.5. Documentation Reading and Updates
*   **Rule:** Before implementing any feature or making changes, you **must** read the relevant documentation in the `docs/` folder.
*   **Rule:** Any code changes that affect functionality described in documentation **must** be accompanied by corresponding updates to the relevant files in `docs/`.
*   **Goal:** Keep documentation and code in sync at all times.

### 1.6. API Documentation
*   **Rule:** After implementing any API endpoint, you **must** add complete documentation for that endpoint in `docs/api.md`.
*   **Required Information:** Request method, path, parameters, request body format, response format, authentication requirements, and example responses.
*   **Language:** All API documentation must be bilingual (English and Chinese).
*   **Rule:** Before implementing or modifying any API endpoint, you **must** reference `docs/api.md` to understand existing API patterns and conventions.
*   **Goal:** Maintain comprehensive and up-to-date API documentation.

### 1.7. Task Completion Tracking
*   **Rule:** After implementing any feature, you **must** mark the corresponding task(s) as completed in `docs/TODO.md` by changing `- [ ]` to `- [x]`.
*   **Goal:** Keep the TODO list accurate and reflect current implementation status.


## 2. Project Structure

### Documentation / 文档结构
- `docs/project-structure.md` - Full project directory tree / 完整项目目录树
- `docs/architecture.md` - Core architecture overview / 核心架构概述
- `docs/configuration.md` - Configuration file details / 配置文件详情
- `docs/api.md` - API endpoints and responses / API 端点和响应
- `docs/database.md` - Database schema / 数据库模式
- `docs/TODO.md` - Implementation tasks / 实现任务清单

### Configuration / 配置
See `docs/configuration.md` for complete configuration details.

Key configuration sections:
- `[server]` - Server settings including `base_url` and `cors_origins`
- `[storage]` - File storage settings including `max_upload_size_mb` (default: 1024MB)
- `[siteinfo]` - Site information forwarded verbatim to the public API (supports `name`, `author`, `avatar`, `bio`, `social_links` array of `{platform, url}` objects, and any additional key-value pairs)

### API Endpoints / API 端点
See `docs/api.md` for complete API documentation.

New public endpoint:
- `GET /api/public/site-info` - Returns site information for frontend display
- `GET /api/public/announcement` - Returns the current site announcement (or null)
- `GET /api/public/tier-descriptions` - Returns all tier descriptions (or null)
