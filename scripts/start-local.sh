#!/bin/bash
set -e
DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DIR"

echo "========================================"
echo "  Snake Skin — Docker Local"
echo "========================================"

if [ ! -f .env ]; then
    echo "[1/3] Generating .env from template..."
    cp deploy/.env.example .env
    echo "  Created .env — edit it to add your API keys"
    echo ""
fi

echo "[2/3] Building & starting containers..."
docker compose -f docker-compose.local.yml up --build -d

echo ""
echo "[3/3] Waiting for services..."
for i in $(seq 1 30); do
    if curl -s http://localhost:8088/api/v1/health > /dev/null 2>&1; then
        echo "  API ready ✓"
        break
    fi
    sleep 1
done

echo ""
echo "========================================"
echo "  Local:  http://localhost:8088"
echo "  Stop:   docker compose -f docker-compose.local.yml down"
echo "  Logs:   docker compose -f docker-compose.local.yml logs -f"
echo "========================================"
