#!/bin/bash
set -e
DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DIR"

echo "========================================"
echo "  Snake Skin — Docker Local"
echo "========================================"

if [ ! -f .env ]; then
    echo "[1/3] 正在从 .env.example 创建 .env..."
    cp .env.example .env
    echo "  ✅ 已创建 .env — 请编辑填入你的 API Key 或 LM Studio 配置"
    echo ""
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
