#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

get_local_ip() {
    local ip
    ip=$(ifconfig en0 2>/dev/null | grep "inet " | awk '{print $2}' | head -1)
    if [ -z "$ip" ]; then
        ip=$(ifconfig en1 2>/dev/null | grep "inet " | awk '{print $2}' | head -1)
    fi
    if [ -z "$ip" ]; then
        ip=$(ifconfig | grep "inet " | grep -v "127.0.0.1" | awk '{print $2}' | head -1)
    fi
    echo "$ip"
}

LOCAL_IP=$(get_local_ip)

if [ -z "$LOCAL_IP" ]; then
    echo "❌ 无法获取本地 IP 地址"
    exit 1
fi

CURRENT_HOST="http://${LOCAL_IP}:1234"
echo "当前本机 IP: $LOCAL_IP"
echo "LM Studio Host: $CURRENT_HOST"

if [ -f .env ]; then
    if grep -q "^LMSTUDIO_HOST=" .env; then
        OLD_HOST=$(grep "^LMSTUDIO_HOST=" .env)
        if [ "$OLD_HOST" = "LMSTUDIO_HOST=$CURRENT_HOST" ]; then
            echo "✓ .env 中的 LMSTUDIO_HOST 已是最新"
        else
            echo "更新 .env 中的 LMSTUDIO_HOST..."
            sed -i '' "s|^LMSTUDIO_HOST=.*|LMSTUDIO_HOST=$CURRENT_HOST|" .env
            echo "✓ 已更新: $OLD_HOST -> LMSTUDIO_HOST=$CURRENT_HOST"
        fi
    else
        echo "添加 LMSTUDIO_HOST 到 .env..."
        echo "LMSTUDIO_HOST=$CURRENT_HOST" >> .env
        echo "✓ 已添加: LMSTUDIO_HOST=$CURRENT_HOST"
    fi
else
    echo "创建 .env 文件..."
    cat > .env << EOF
LMSTUDIO_HOST=$CURRENT_HOST
LMSTUDIO_MODEL=qwen/qwen3.6-27b
EOF
    echo "✓ 已创建 .env"
fi

echo ""
echo "========================================"
echo "提示：每次切换网络后，请运行此脚本更新 IP"
echo "或者直接运行: ./update-ip.sh && docker compose -f docker-compose.local.yml up -d api"
echo "========================================"
