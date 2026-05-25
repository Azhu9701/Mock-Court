#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "============================================"
echo "  万民幡 · 部署准备"
echo "============================================"
echo ""

# ── 1. 检查 Docker ──
if ! command -v docker &>/dev/null; then
    echo "❌ 未安装 Docker。请先安装: https://docs.docker.com/engine/install/"
    exit 1
fi
echo "✓ Docker 已安装"

if ! docker compose version &>/dev/null; then
    echo "❌ Docker Compose 不可用"
    exit 1
fi
echo "✓ Docker Compose 已安装"
echo ""

# ── 2. 生成 .env ──
if [ -f .env ]; then
    echo "⚠ .env 已存在，跳过生成。如需重新生成请删除 .env 后重试。"
else
    echo "--- 域名配置 ---"
    read -p "你的域名（如 banner.example.com）: " DOMAIN
    DOMAIN=${DOMAIN:-example.com}

    echo ""
    echo "--- Tinyauth 管理员 ---"
    read -p "管理员用户名 [admin]: " ADMIN_USER
    ADMIN_USER=${ADMIN_USER:-admin}
    read -sp "管理员密码: " ADMIN_PASS
    echo ""

    # Generate bcrypt hash
    if command -v htpasswd &>/dev/null; then
        ADMIN_HASH=$(htpasswd -nbB "" "$ADMIN_PASS" | cut -d: -f2)
    else
        # Fallback: use python bcrypt (already checked earlier)
        ADMIN_HASH=$(printf '%s' "$ADMIN_PASS" | python3 -c "import bcrypt, sys; print(bcrypt.hashpw(sys.stdin.read().encode(), bcrypt.gensalt()).decode())" 2>/dev/null || echo "MANUAL_HASH_NEEDED")
    fi

    # Generate random secret
    SECRET=$(python3 -c "import secrets; print(secrets.token_hex(16))" 2>/dev/null || echo "change-me-$(date +%s)")

    echo ""
    echo "--- Agent Proxy 中转站 ---"
    read -p "中转站 URL []: " RELAY_URL
    RELAY_URL=${RELAY_URL:-}
    read -sp "中转站 API Key: " RELAY_KEY
    echo ""

    echo ""
    echo "--- OAuth（可选，用于用户自助注册）---"
    read -p "GitHub Client ID（回车跳过）: " GITHUB_ID
    read -sp "GitHub Client Secret（回车跳过）: " GITHUB_SECRET
    echo ""

    cat > .env << EOF
# ── 万民幡部署配置 ──
# 生成时间: $(date)

DOMAIN=$DOMAIN

TINYAUTH_SECRET=$SECRET
TINYAUTH_ADMIN_USER=$ADMIN_USER
TINYAUTH_ADMIN_PASSWORD_HASH=$ADMIN_HASH
TINYAUTH_GITHUB_CLIENT_ID=$GITHUB_ID
TINYAUTH_GITHUB_CLIENT_SECRET=$GITHUB_SECRET

AI_RELAY_URL=$RELAY_URL
AGENT_PROXY_KEY=$RELAY_KEY
EOF

    echo ""
    echo "✓ .env 已生成"
    if [ "$ADMIN_HASH" = "MANUAL_HASH_NEEDED" ]; then
        echo "⚠  bcrypt hash 生成失败。请手动替换 .env 中的 TINYAUTH_ADMIN_PASSWORD_HASH"
        echo "   生成命令: htpasswd -nbB '' 'yourpassword' | cut -d: -f2"
    fi
fi
echo ""

# ── 3. 检查 DNS ──
if [ -f .env ]; then
    source .env 2>/dev/null || true
    echo "--- DNS 检查 ---"
    for sub in auth app api; do
        HOST="${sub}.${DOMAIN}"
        if host "$HOST" >/dev/null 2>&1; then
            echo "  ✓ $HOST → $(host $HOST | grep 'has address' | head -1 | awk '{print $NF}')"
        else
            echo "  ⚠ $HOST 未解析 —— 部署前请在 DNS 中添加 A 记录指向服务器 IP"
        fi
    done
fi
echo ""

# ── 4. 构建 Docker 镜像 ──
echo "--- 构建镜像 ---"
read -p "现在就构建吗？(y/N): " BUILD
if [ "$BUILD" = "y" ] || [ "$BUILD" = "Y" ]; then
    docker compose build
    echo "✓ 构建完成"
fi
echo ""

echo "============================================"
echo "  部署准备完成"
echo ""
echo "  启动服务:"
echo "    docker compose up -d"
echo ""
echo "  查看日志:"
echo "    docker compose logs -f"
echo ""
echo "  停止服务:"
echo "    docker compose down"
echo "============================================"
