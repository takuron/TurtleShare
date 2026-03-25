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


## 2. Project Structure

No details available; to be supplemented and refined.
