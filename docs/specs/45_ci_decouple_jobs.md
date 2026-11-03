# Especificação Formal: Desacoplamento de Jobs na Esteira CI/CD (#45)

## 1. Contexto e Motivação
A esteira do repositório `navalha` vinha apresentando falha por estouro do tempo limite (timeout de 45m do runner Forgejo) durante o job de release. O problema ocorria porque o job `deploy-banca` acumulava tanto a compilação completa do bundle WebAssembly (`navalha-web`) com deploy no Cloudflare Workers quanto a compilação cruzada para 3 arquiteturas de binários (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-gnu`), além de uploads no Cloudflare R2 e rotação na Banca.

Adicionalmente, havia redundância na compilação do bundle WASM e uso de terminologia imperial em desacordo com as diretrizes do ecossistema.

## 2. Requisitos da Especificação

### 2.1 Desacoplamento Estrutural de Jobs
O arquivo `.forgejo/workflows/ci.yml` deve conter três jobs principais, desacoplados e com responsabilidades delimitadas:
1. `test` ("Checks, Tests & Quality Gates"):
   - Executa formatação (`cargo fmt -- --check`), linter (`cargo clippy --all-targets -- -D warnings`), testes da workspace (`cargo test --workspace`) e auditorias de qualidade.
   - Não deve duplicar etapas de publicação.

2. `deploy-web` ("Build WebAssembly & Deploy to Cloudflare Workers"):
   - Dependência estrita: `needs: [test]`.
   - Condição: `success() && (github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')) && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')`.
   - Executa a compilação exclusiva do pacote WASM de `crates/navalha-web` com `wasm-pack build --target web --out-dir ../../site/pkg`.
   - Executa a publicação no Cloudflare Workers via Wrangler (`navalha.2lp.in`).

3. `deploy-banca` ("Build & Publish Navalha Multiarch to Banca"):
   - Dependência estrita: `needs: [test]`.
   - Condição: idêntica ao `deploy-web`.
   - Focado exclusivamente na compilação dos binários CLI (`navalha-cli`) para as três arquiteturas (`x86_64-musl`, `aarch64-musl`, `x86_64-pc-windows-gnu`).
   - Empacota os arquivos em `dist/packages/`, gera `SHA256SUMS`, faz upload para o Cloudflare R2 e executa rotação no manifesto da Banca.

### 2.2 Purificação Terminológica e Credenciais
- Proibido o uso de termos relacionados a imperialismos como "soberano" ou "soberana".
- Configurações de registro de pacotes e autenticação devem referenciar "Canônico" / "Gambi Cargo Registry".
- Unificação do fallback de tokens: `GAMBI_2LP_IN_TOKEN || GIT_2LP_TOKEN || REGISTRY_TOKEN || LUMEN_REGISTRY_TOKEN || FORGEJO_TOKEN` com saneamento de prefixo `Bearer`.
