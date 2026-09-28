#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "⚡ [navalha] Compilando motor e ferramentas Navalha..."

# 1. Compilação do Core e CLI Nativa
echo "🔨 Compilando binário nativo release (navalha-cli)..."
cargo build --release -p navalha-cli

mkdir -p "$PROJECT_ROOT/dist/bin"
cp "$PROJECT_ROOT/target/release/navalha" "$PROJECT_ROOT/dist/bin/navalha"
chmod +x "$PROJECT_ROOT/dist/bin/navalha"
echo "✅ Binário CLI gerado em: dist/bin/navalha"

# 2. Compilação da WebAssembly (navalha-web) e montagem de site/
echo "🌐 Compilando WebAssembly com wasm-pack (navalha-web)..."
mkdir -p "$PROJECT_ROOT/dist/web" "$PROJECT_ROOT/site/pkg"
(
    cd "$PROJECT_ROOT/crates/navalha-web"
    wasm-pack build --target web --out-dir "$PROJECT_ROOT/site/pkg"
    cp -r "$PROJECT_ROOT/site/pkg" "$PROJECT_ROOT/dist/web/"
    cp -r assets "$PROJECT_ROOT/dist/web/"
    cp -r assets "$PROJECT_ROOT/site/"
    cp index.html "$PROJECT_ROOT/dist/web/"
)
echo "✅ Aplicação Web Wasm gerada em: site/ e dist/web/"
echo "⚡ [navalha] Build completo finalizado com sucesso!"
