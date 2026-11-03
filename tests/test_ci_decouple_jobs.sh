#!/usr/bin/env bash
# ==============================================================================
# TESTE: Desacoplamento de Jobs e Purificação da Esteira CI/CD (#45)
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CI_FILE="$ROOT_DIR/.forgejo/workflows/ci.yml"
SPEC_FILE="$ROOT_DIR/docs/specs/45_ci_decouple_jobs.md"

echo "==> [TEST] Validando conformidade da esteira desacoplada (#45)..."

FAILED=0

# 1. Validar existência da especificação
if [[ ! -f "$SPEC_FILE" ]]; then
    echo "  [FAIL] Arquivo de especificação ausente: $SPEC_FILE"
    FAILED=1
else
    echo "  [PASS] Especificação formal presente."
fi

# 2. Validar existência do ci.yml
if [[ ! -f "$CI_FILE" ]]; then
    echo "  [FAIL] Arquivo de workflow ausente: $CI_FILE"
    exit 1
fi

# 3. Validar ausência de termos imperiais ("soberano", "soberana")
if grep -iE "(soberan[oa])" "$CI_FILE" >/dev/null 2>&1; then
    echo "  [FAIL] Termo imperial detectado em $CI_FILE"
    FAILED=1
else
    echo "  [PASS] Esteira livre de terminologia imperial."
fi

# 4. Validar existência dos três jobs desacoplados: test, deploy-web, deploy-banca
if grep -E '^[[:space:]]{2}test:' "$CI_FILE" >/dev/null 2>&1; then
    echo "  [PASS] Job 'test' declarado."
else
    echo "  [FAIL] Job 'test' ausente."
    FAILED=1
fi

if grep -E '^[[:space:]]{2}deploy-web:' "$CI_FILE" >/dev/null 2>&1; then
    echo "  [PASS] Job 'deploy-web' declarado."
else
    echo "  [FAIL] Job 'deploy-web' ausente."
    FAILED=1
fi

if grep -E '^[[:space:]]{2}deploy-banca:' "$CI_FILE" >/dev/null 2>&1; then
    echo "  [PASS] Job 'deploy-banca' declarado."
else
    echo "  [FAIL] Job 'deploy-banca' ausente."
    FAILED=1
fi

# 5. Validar que deploy-banca NÃO compila navalha-web / wasm-pack
DEPLOY_BANCA_SECTION=$(awk '/^[[:space:]]{2}deploy-banca:/{flag=1; next} /^[[:space:]]{2}[a-zA-Z0-9_-]+:/{flag=0} flag' "$CI_FILE")

if echo "$DEPLOY_BANCA_SECTION" | grep -iE '(wasm-pack|navalha-web|wrangler deploy)' >/dev/null 2>&1; then
    echo "  [FAIL] deploy-banca ainda contém compilação WASM ou deploy wrangler acoplado."
    FAILED=1
else
    echo "  [PASS] deploy-banca desacoplado de WASM e wrangler."
fi

# 6. Validar que deploy-web contém compilação wasm-pack e wrangler deploy
DEPLOY_WEB_SECTION=$(awk '/^[[:space:]]{2}deploy-web:/{flag=1; next} /^[[:space:]]{2}[a-zA-Z0-9_-]+:/{flag=0} flag' "$CI_FILE")

if echo "$DEPLOY_WEB_SECTION" | grep -E 'wasm-pack build --target web' >/dev/null 2>&1 && \
   echo "$DEPLOY_WEB_SECTION" | grep -E 'wrangler deploy' >/dev/null 2>&1; then
    echo "  [PASS] deploy-web contém compilação WASM e deploy wrangler."
else
    echo "  [FAIL] deploy-web ausente de compilação WASM ou deploy wrangler."
    FAILED=1
fi

if [[ $FAILED -ne 0 ]]; then
    echo "❌ [RED TEST] Auditoria falhou conforme esperado na Fase 4."
    exit 1
fi

echo "✔ [GREEN TEST] Validação estrutural da esteira aprovada com sucesso."
