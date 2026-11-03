#!/usr/bin/env bash
# ==============================================================================
# TESTE: Desacoplamento de Jobs e Purificação da Esteira CI/CD (#45)
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CI_FILE="$ROOT_DIR/.forgejo/workflows/ci.yml"
SPEC_FILE="$ROOT_DIR/docs/specs/45_ci_decouple_jobs.md"

echo "==> [TEST] Validando conformidade da esteira desacoplada (#45)..."

audit_workflow_decoupling() {
    local file="$1"
    local failed=0

    # 1. Ausência de termos imperiais
    if grep -iE "(soberan[oa])" "$file" >/dev/null 2>&1; then
        echo "  [FAIL] Termo imperial detectado em $(basename "$file")"
        failed=1
    fi

    # 2. Presença dos três jobs
    for job in "test" "deploy-web" "deploy-banca"; do
        if ! grep -E "^[[:space:]]{2}${job}:" "$file" >/dev/null 2>&1; then
            echo "  [FAIL] Job '${job}' ausente em $(basename "$file")"
            failed=1
        fi
    done

    # 3. deploy-banca não pode conter wasm-pack ou wrangler deploy
    local banca_section
    banca_section=$(awk '/^[[:space:]]{2}deploy-banca:/{flag=1; next} /^[[:space:]]{2}[a-zA-Z0-9_-]+:/{flag=0} flag' "$file")
    if echo "$banca_section" | grep -iE '(wasm-pack|navalha-web|wrangler deploy)' >/dev/null 2>&1; then
        echo "  [FAIL] deploy-banca acoplado indevidamente a WASM/Wrangler"
        failed=1
    fi

    # 4. deploy-web deve conter wasm-pack e wrangler deploy
    local web_section
    web_section=$(awk '/^[[:space:]]{2}deploy-web:/{flag=1; next} /^[[:space:]]{2}[a-zA-Z0-9_-]+:/{flag=0} flag' "$file")
    if ! echo "$web_section" | grep -E 'wasm-pack build --target web' >/dev/null 2>&1 || \
       ! echo "$web_section" | grep -E 'wrangler deploy' >/dev/null 2>&1; then
        echo "  [FAIL] deploy-web ausente de wasm-pack ou wrangler deploy"
        failed=1
    fi

    return "$failed"
}

# 1. Validar existência da especificação
if [[ ! -f "$SPEC_FILE" ]]; then
    echo "  [FAIL] Arquivo de especificação ausente: $SPEC_FILE"
    exit 1
else
    echo "  [PASS] Especificação formal presente."
fi

# 2. Validar existência do ci.yml
if [[ ! -f "$CI_FILE" ]]; then
    echo "  [FAIL] Arquivo de workflow ausente: $CI_FILE"
    exit 1
fi

if ! audit_workflow_decoupling "$CI_FILE"; then
    echo "❌ [FAIL] Falha na conformidade do arquivo de workflow real."
    exit 1
fi

echo "✔ [GREEN TEST] Validação estrutural da esteira aprovada com sucesso."

echo "=== [MUTATION] Testes de Mutação da Esteira Desacoplada ==="
TMP_MUTANT=$(mktemp --suffix=.yml)
trap 'rm -f "$TMP_MUTANT"' EXIT

# Mutante 1: Termo imperial injetado
cp "$CI_FILE" "$TMP_MUTANT"
echo "# Repositório soberano" >> "$TMP_MUTANT"
if audit_workflow_decoupling "$TMP_MUTANT" >/dev/null 2>&1; then
    echo "  [FAIL] Mutante 1 (termo imperial) não foi detectado!"
    exit 1
else
    echo "  [PASS] Mutante 1 (termo imperial) detectado e rejeitado"
fi

# Mutante 2: deploy-banca contendo wasm-pack
cp "$CI_FILE" "$TMP_MUTANT"
sed -i '/deploy-banca:/a \    steps:\n      - run: wasm-pack build --target web' "$TMP_MUTANT"
if audit_workflow_decoupling "$TMP_MUTANT" >/dev/null 2>&1; then
    echo "  [FAIL] Mutante 2 (acoplamento wasm-pack em deploy-banca) não foi detectado!"
    exit 1
else
    echo "  [PASS] Mutante 2 (acoplamento wasm-pack) detectado e rejeitado"
fi

# Mutante 3: ausência do job deploy-web
cp "$CI_FILE" "$TMP_MUTANT"
sed -i 's/deploy-web:/deleted-job:/' "$TMP_MUTANT"
if audit_workflow_decoupling "$TMP_MUTANT" >/dev/null 2>&1; then
    echo "  [FAIL] Mutante 3 (ausência de deploy-web) não foi detectado!"
    exit 1
else
    echo "  [PASS] Mutante 3 (ausência de deploy-web) detectado e rejeitado"
fi

# Mutante 4: deploy-web sem wrangler deploy
cp "$CI_FILE" "$TMP_MUTANT"
sed -i 's/wrangler deploy/echo deploy/' "$TMP_MUTANT"
if audit_workflow_decoupling "$TMP_MUTANT" >/dev/null 2>&1; then
    echo "  [FAIL] Mutante 4 (deploy-web sem wrangler deploy) não foi detectado!"
    exit 1
else
    echo "  [PASS] Mutante 4 (deploy-web sem wrangler deploy) detectado e rejeitado"
fi

echo "✔ [MUTATION TEST] Todos os 4 mutantes foram interceptados com sucesso."
