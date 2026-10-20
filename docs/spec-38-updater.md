# Especificação Técnica: Auto-Atualizador & Distribuição na Banca (Ticket #38)

## 1. Contexto e Motivação
Em conformidade com a norma GOST v1.5.0, a compilação de binários nativos de release e desenvolvimento é restrita à esteira sequencial (Forgejo Runner). A distribuição oficial ocorre através da vitrine soberana da Banca (`https://banca.2lp.in/downloads/navalha/`), com retenção estrita de 5 versões estáveis e 10 compilações de desenvolvimento (nightly).
O binário do `navalha` passa a incorporar funcionalidade nativa de auto-atualização atômica e verificação de novas versões, consumindo diretamente esses canais.

## 2. Endpoints Soberanos da Banca
O módulo de atualização resolve os seguintes endpoints:
- **Canal Estável (Produção)**:
  - Binário mais recente: `https://banca.2lp.in/downloads/navalha/stable-latest/navalha`
  - Identificador de versão: `https://banca.2lp.in/downloads/navalha/version`
- **Canal Nightly (Desenvolvimento)**:
  - Binário mais recente: `https://banca.2lp.in/downloads/navalha/nightly-latest/navalha`
  - Identificador de versão: `https://banca.2lp.in/downloads/navalha/nightly-version`
- **Versão Fixada (Pin)**:
  - `https://banca.2lp.in/downloads/navalha/releases/v{pin}/navalha`

Nenhum fallback para releases do Codeberg ou GitHub é executado, mantendo a soberania da infraestrutura.

## 3. Persistência de Configuração e Canal Padrão
Para permitir que o usuário defina seu canal de preferência (por exemplo, `nightly` para receber compilações frequentes ou `stable` para lançamentos validados):
- **Arquivo de Configuração**: `$XDG_CONFIG_HOME/navalha/config.toml` (fallback `~/.config/navalha/config.toml`).
  ```toml
  [update]
  channel = "nightly" # ou "stable"
  ```
- **Gravação do Canal**:
  - `navalha update --channel nightly --save` grava `channel = "nightly"` no arquivo de configuração do usuário.
  - `navalha config set channel <stable|nightly>` permite configurar diretamente o canal padrão.
- **Ordem de Precedência**:
  1. Flag de linha de comando: `--channel <stable|nightly>`
  2. Variável de ambiente: `NAVALHA_CHANNEL`
  3. Arquivo de configuração do usuário: `~/.config/navalha/config.toml`
  4. Valor padrão de fallback: `"stable"`

## 4. Comandos e Interface da CLI
O `navalha` adiciona os seguintes subcomandos:
- `navalha update`:
  - `--channel <stable|nightly>`: Seleciona temporariamente o canal de atualização.
  - `--save`: Salva o canal informado como o padrão do usuário.
  - `--pin <versão>`: Instala ou trava em uma versão estável específica.
  - `--check`: Apenas checa se há atualização disponível sem baixar.
  - `--target <caminho>`: Sobrescreve o caminho do executável de destino.
- `navalha check-update`:
  - `--channel <stable|nightly>`: Checa disponibilidade no canal especificado.
- `navalha config set <chave> <valor>`:
  - Permite definir chaves de configuração como `channel`.

## 5. Algoritmo de Substituição Atômica
1. O executável determina o caminho do binário atual via `std::env::current_exe()`.
2. O novo binário é baixado em um arquivo temporário no mesmo sistema de arquivos (`<current_exe>.tmp.<random>`).
3. As permissões de execução (`chmod 0o755`) são atribuídas no arquivo temporário.
4. É executada uma substituição atômica via `std::fs::rename`. Caso o arquivo esteja em uso no Linux, ele desassocia o inode antigo e associa o novo sem interromper a execução do processo chamador.

## 6. Pipeline de Entrega na Esteira
No workflow de CI (`.forgejo/workflows/ci.yml`), a etapa de build nativo publica os artefatos compilados na rota `/srv/banca/downloads/navalha/` e no R2 da Banca, atualizando os links `stable-latest` ou `nightly-latest` e o arquivo `version` correspondente.
