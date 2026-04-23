console.log('检查 Node.js 环境...');
console.log('Node.js 版本:', process.version);

const fs = require('fs');

// 尝试使用 Node.js 内置的 crypto 模块
// 注意：Node.js 内置不支持 Argon2，需要安装 argon2 包

console.log('\n尝试生成密码哈希...');
console.log('由于 Node.js 内置不支持 Argon2，需要安装 argon2 包。');
console.log('\n请在项目目录下运行以下命令之一:');
console.log('');
console.log('方案 1: 安装 argon2 包 (需要编译原生模块)');
console.log('  npm install argon2');
console.log('  然后运行: node gen_hash_node.js');
console.log('');
console.log('方案 2: 使用在线 Argon2 哈希生成器');
console.log('  访问: https://argon2.online/');
console.log('  使用以下参数:');
console.log('    - 算法: Argon2id');
console.log('    - 时间开销 (t): 2');
console.log('    - 内存开销 (m): 19456 KB (19 MB)');
console.log('    - 并行度 (p): 1');
console.log('    - 哈希长度: 32 字节');
console.log('    - 密码: admin123');
console.log('');
console.log('方案 3: 安装 Rust 工具链');
console.log('  安装 Rust: https://rustup.rs/');
console.log('  然后运行: cargo run -- hash-pw admin123');
console.log('');

// 预先计算好的几个可能的哈希值 (使用相同密码和参数，但不同盐值)
// 这些是为了演示，实际使用时需要自己生成
const sampleHashes = [
  {
    description: '示例哈希 1 (admin123)',
    hash: '$argon2id$v=19$m=19456,t=2,p=1$FVGhfDUHIQpSCUabKbhkVA$e0tpWtkmWL7uKmX2t517HOAHpUuBbmIpluFwDv522Ns'
  }
];

console.log('\n配置文件中现有的密码哈希:');
console.log('  ', sampleHashes[0].hash);
console.log('');
console.log('如果这个哈希值与 "admin123" 不匹配，请使用上述方案之一生成新的哈希值。');
console.log('');
console.log('生成新哈希值后，将 config.toml 中的 password_hash 替换为新的哈希值。');
