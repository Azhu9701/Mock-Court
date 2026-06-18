#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

get_local_ip() {
    local ip=""
    # macOS
    ip=$(ipconfig getifaddr en0 2>/dev/null || true)
    if [ -z "$ip" ]; then
        ip=$(ipconfig getifaddr en1 2>/dev/null || true)
    fi
    # Linux
    if [ -z "$ip" ]; then
        ip=$(hostname -I 2>/dev/null | awk '{print $1}' || true)
    fi
    # 通用 fallback
    if [ -z "$ip" ]; then
        ip=$(ifconfig 2>/dev/null | grep "inet " | grep -v "127.0.0.1" | awk '{print $2}' | head -1 || true)
    fi
    # Linux ip 命令
    if [ -z "$ip" ]; then
        ip=$(ip route get 1 2>/dev/null | awk '{print $7; exit}' || true)
    fi
    echo "$ip"
}

LOCAL_IP=$(get_local_ip)

if [ -z "$LOCAL_IP" ]; then
    echo "⚠️ 无法获取本地 IP 地址，跳过更新（不影响 Docker 内通过 host.docker.internal 连接）"
    exit 0
fi

CURRENT_HOST="http://${LOCAL_IP}:1234"
echo "当前本机 IP: $LOCAL_IP"
echo "LM Studio Host: $CURRENT_HOST"

if [ -f .env ]; then
    # 跨平台 sed：先检测 GNU sed 还是 BSD sed
    if sed --version 2>/dev/null | grep -q GNU; then
        SED_INPLACE="sed -i"
    else
        SED_INPLACE="sed -i ''"
    fi

    if grep -q "^LMSTUDIO_HOST=" .env; then
        OLD_HOST=$(grep "^LMSTUDIO_HOST=" .env)
        if [ "$OLD_HOST" = "LMSTUDIO_HOST=$CURRENT_HOST" ]; then
            echo "✅ .env 中的 LMSTUDIO_HOST 已是最新"
        else
            echo "更新 .env 中的 LMSTUDIO_HOST..."
            $SED_INPLACE "s|^LMSTUDIO_HOST=.*|LMSTUDIO_HOST=$CURRENT_HOST|" .env
            echo "✅ 已更新: $OLD_HOST -> LMSTUDIO_HOST=$CURRENT_HOST"
        fi
    else
        echo "添加 LMSTUDIO_HOST 到 .env..."
        echo "LMSTUDIO_HOST=$CURRENT_HOST" >> .env
        echo "✅ 已添加: LMSTUDIO_HOST=$CURRENT_HOST"
    fi
else
    echo "创建 .env 文件..."
    cat > .env << EOF
LMSTUDIO_HOST=$CURRENT_HOST
LMSTUDIO_MODEL=qwen/qwen3.5-9b
EOF
    echo "✅ 已创建 .env"
fi

echo ""
echo "========================================"
echo "提示：每次切换网络后，请运行此脚本更新 IP"
echo "或者直接运行: ./update-ip.sh && docker compose -f docker-compose.local.yml up -d api"
echo "========================================"
