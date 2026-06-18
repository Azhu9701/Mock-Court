#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "  Snake Skin - 本地开发环境启动"
echo "========================================"
echo ""

# ── 前置检查 ──
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ 未找到 $1，请先安装"
        echo "   Docker 安装指南: https://docs.docker.com/get-docker/"
        exit 1
    fi
}

check_command docker
check_command docker\ compose

# 检查 .env 文件
if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        echo "⚠️  .env 文件不存在，正在从 .env.example 创建..."
        cp .env.example .env
        echo "✅ 已创建 .env，请编辑填入你的 API Key 或 LM Studio 配置后再启动"
        echo ""
        echo "   快速配置（二选一）："
        echo "   1) 本地模型: LMSTUDIO_HOST=http://host.docker.internal:1234"
        echo "   2) 云端 API: OPENAI_API_KEY=sk-... / DEEPSEEK_API_KEY=sk-..."
        echo ""
        exit 1
    else
        echo "❌ .env 和 .env.example 均不存在，无法启动"
        exit 1
    fi
fi

# 更新 LM Studio IP（仅当用户配置了 LM Studio 时）
if [ -f ./update-ip.sh ] && grep -q "^LMSTUDIO_HOST=" .env 2>/dev/null; then
    echo "🔄 更新 LM Studio 连接地址..."
    bash ./update-ip.sh || true
fi

echo ""
echo "🚀 正在启动服务..."
docker compose -f docker-compose.local.yml up --build -d

echo ""
echo "⏳ 等待服务就绪..."
sleep 5

# 检查 API 健康状态
for i in $(seq 1 30); do
    if curl -s http://localhost:8088/api/v1/health > /dev/null 2>&1; then
        echo "✅ API 服务已就绪"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "⚠️ API 启动超时，请查看日志: docker compose -f docker-compose.local.yml logs -f api"
        exit 1
    fi
    sleep 1
done

echo ""
echo "========================================"
echo "  ✅ 服务启动完成！"
echo ""
echo "  访问地址: http://localhost:8088"
echo "  API 地址: http://localhost:8088/api/v1"
echo ""
echo "  常用命令："
echo "    查看日志: docker compose -f docker-compose.local.yml logs -f"
echo "    停止服务: docker compose -f docker-compose.local.yml down"
echo "    重新构建: docker compose -f docker-compose.local.yml up --build -d"
echo "========================================"
