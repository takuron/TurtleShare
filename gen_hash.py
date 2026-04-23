import sys

print("检查 Python 环境...")
print(f"Python 版本: {sys.version}")

try:
    from argon2.low_level import hash_secret, Type
    print("argon2-cffi 已安装")
except ImportError as e:
    print(f"argon2-cffi 未安装，错误: {e}")
    print("正在尝试安装 argon2-cffi...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "argon2-cffi"])
    print("安装完成")
    from argon2.low_level import hash_secret, Type

password = "admin123".encode("utf-8")
print(f'生成密码 "admin123" 的哈希值...')

hash_result = hash_secret(
    secret=password,
    salt=None,
    time_cost=2,
    memory_cost=19456,
    parallelism=1,
    hash_len=32,
    type=Type.ID,
)

hash_str = hash_result.decode("ascii")
print("=" * 60)
print("生成的密码哈希值:")
print("=" * 60)
print("")
print(hash_str)
print("")
print("=" * 60)
print("请将以下行复制到 config.toml 的 [admin] 部分:")
print("=" * 60)
print("")
print(f'password_hash = "{hash_str}"')
print("")

with open("generated_password_hash.txt", "w", encoding="utf-8") as f:
    f.write(f"密码: admin123\n")
    f.write(f"哈希值: {hash_str}\n")
    f.write(f"\n")
    f.write(f"在 config.toml 中使用:\n")
    f.write(f'password_hash = "{hash_str}"\n')

print(f"哈希值已保存到: generated_password_hash.txt")
