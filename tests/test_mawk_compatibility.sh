#!/usr/bin/env bash
# ==============================================================================
# TESTE: Verificação de Compatibilidade com mawk (#47)
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_SCRIPT="$ROOT_DIR/tests/test_ci_decouple_jobs.sh"
SPEC_FILE="$ROOT_DIR/docs/specs/47_mawk_compatibility.md"

echo "==> [TEST] Validando ausência de quantificadores de intervalo no script de teste (#47)..."

audit_mawk_syntax() {
    local file="$1"
    if grep -E "awk.*\{[0-9]+(,[0-9]*)?\}" "$file" >/dev/null 2>&1; then
        return 1
    fi
    return 0
}

# 1. Validar existência da especificação
if [[ ! -f "$SPEC_FILE" ]]; then
    echo "  [FAIL] Arquivo de especificação ausente: $SPEC_FILE"
    exit 1
fi

# 2. Validar que não há quantificadores de intervalo {n} ou {n,m} em comandos awk
if ! audit_mawk_syntax "$TEST_SCRIPT"; then
    echo "  [FAIL] Quantificador de intervalo incompatível com mawk detectado em awk!"
    exit 1
else
    echo "  [PASS] Script livre de quantificadores de intervalo no awk."
fi

echo "✔ [GREEN TEST] Compatibilidade de sintaxe com mawk aprovada."

echo "=== [MUTATION] Testes de Mutação de Sintaxe mawk ==="
TMP_MUTANT=$(mktemp --suffix=.sh)
trap 'rm -f "$TMP_MUTANT"' EXIT

# Mutante 1: Intervalo exato {2} em comando awk
echo 'awk "/^[[:space:]]{2}/" file' > "$TMP_MUTANT"
if audit_mawk_syntax "$TMP_MUTANT"; then
    echo "  [FAIL] Mutante 1 (quantificador exato {2}) não foi interceptado!"
    exit 1
else
    echo "  [PASS] Mutante 1 (quantificador exato {2}) interceptado e rejeitado"
fi

# Mutante 2: Intervalo variável {1,3} em comando awk
echo 'awk "/^[[:space:]]{1,3}/" file' > "$TMP_MUTANT"
if audit_mawk_syntax "$TMP_MUTANT"; then
    echo "  [FAIL] Mutante 2 (quantificador variável {1,3}) não foi interceptado!"
    exit 1
else
    echo "  [PASS] Mutante 2 (quantificador variável {1,3}) interceptado e rejeitado"
fi

echo "✔ [MUTATION TEST] Todos os mutantes interceptados com sucesso."
