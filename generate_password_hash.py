import sys

def install_argon2():
    import subprocess
    print("正在安装 argon2-cffi 库...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "argon2-cffi"])
    print("安装完成！\n")

try:
    from argon2.low_level import hash_secret, Type
except ImportError:
    install_argon2()
    from argon2.low_level import hash_secret, Type

def generate_argon2_hash(password: str) -> str:
    password_bytes = password.encode('utf-8')
    
    hash_result = hash_secret(
        secret=password_bytes,
        salt=None,
        time_cost=2,
        memory_cost=19456,
        parallelism=1,
        hash_len=32,
        type=Type.ID,
    )
    
    return hash_result.decode('ascii')

if __name__ == "__main__":
    print("=" * 60)
    print("Argon2 密码哈希生成器")
    print("=" * 60)
    print("\n参数配置:")
    print("  - 算法: Argon2id")
    print("  - 时间开销 (t_cost): 2")
    print("  - 内存开销 (m_cost): 19456 KB (19MB)")
    print("  - 并行度 (p_cost): 1")
    print("  - 哈希长度: 32 字节")
    print("")
    
    if len(sys.argv) > 1:
        password = sys.argv[1]
        print(f"密码: {password}")
    else:
        password = input("请输入要哈希的密码: ")
    
    if not password:
        print("\n错误: 密码不能为空！")
        sys.exit(1)
    
    print("\n正在生成哈希值，请稍候...\n")
    
    hash_value = generate_argon2_hash(password)
    
    print("=" * 60)
    print("生成结果:")
    print("=" * 60)
    print(f"\n原始密码: {password}")
    print(f"\nArgon2id 哈希值 (PHC 格式):")
    print(f"{hash_value}")
    print("\n" + "=" * 60)
    print("使用方法:")
    print("=" * 60)
    print(f"\n将以下行复制到 config.toml 文件的 [admin] 部分:")
    print(f"\n  password_hash = \"{hash_value}\"")
    print("")
