#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${PORT:-8080}"
WEB_DIR="$PROJECT_ROOT/crates/navalha-web"

echo "======================================================================"
echo " ★ NAVALHA — Servidor de Desenvolvimento Local (Web & Wasm) ★"
echo "======================================================================"

# Verifica se o pacote Wasm já foi compilado
if [ ! -f "$WEB_DIR/dist/pkg/navalha_web.js" ]; then
    echo "📦 Pacote Wasm não encontrado. Compilando com wasm-pack..."
    (cd "$WEB_DIR" && wasm-pack build --target web --out-dir dist/pkg)
fi

# Cria symlink de conveniência em crates/navalha-web/pkg apontando para dist/pkg se necessário
mkdir -p "$WEB_DIR/pkg"
cp -r "$WEB_DIR/dist/pkg/"* "$WEB_DIR/pkg/" 2>/dev/null || true

echo "🚀 Iniciando servidor HTTP em http://localhost:$PORT..."
echo "   Pressione Ctrl+C para encerrar."
python3 -m http.server "$PORT" --directory "$WEB_DIR"
