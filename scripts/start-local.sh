#!/bin/bash
set -e
DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DIR"

echo "========================================"
echo "  Snake Skin — Docker Local"
echo "========================================"

if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        echo "[1/3] 正在从 .env.example 创建 .env..."
        cp .env.example .env
        echo "  ✅ 已创建 .env — 请编辑填入你的 API Key 或 LM Studio 配置后再启动"
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

echo "[2/3] 构建并启动容器..."
docker compose -f docker-compose.local.yml up --build -d

echo ""
echo "[3/3] 等待服务就绪..."
for i in $(seq 1 30); do
    if curl -s http://localhost:8088/api/v1/health > /dev/null 2>&1; then
        echo "  ✅ API 已就绪"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "  ⚠️ API 启动超时，请查看日志: docker compose -f docker-compose.local.yml logs -f api"
    fi
    sleep 1
done

echo ""
echo "========================================"
echo "  访问:  http://localhost:8088"
echo "  停止:  docker compose -f docker-compose.local.yml down"
echo "  日志:  docker compose -f docker-compose.local.yml logs -f"
echo "========================================"
