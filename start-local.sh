#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "  万民幡 - 本地开发环境启动"
echo "========================================"
echo ""

./update-ip.sh

echo ""
echo "正在启动服务..."
docker compose -f docker-compose.local.yml up -d

echo ""
echo "========================================"
echo "  服务启动完成！"
echo ""
echo "  访问地址: http://localhost:8088"
echo "  查看日志: docker compose -f docker-compose.local.yml logs -f"
echo "========================================"
