# Especificação Técnica: Auto-Atualizador OpenDesktop XDG e Distribuição Soberana

**Ticket:** #38  
**Componente:** `crates/navalha-cli`  
**Status:** Aprovado para Implementação  
**Padrão:** OpenDesktop Base Directory Specification (XDG) & Norma GOST  

---

## 1. Contexto e Motivação

O `navalha` adota a infraestrutura soberana da Banca (`https://banca.2lp.in/downloads/navalha/`) para entrega de binários pré-compilados em canais estável e nightly. Para assegurar total alinhamento com os demais aplicativos do ecossistema soberano (`ishcondidim`, `papeli`, `cccp`), o gerenciador de atualizações e configurações do Navalha deve adotar estritamente o padrão OpenDesktop XDG, permitindo fixação de versões (*pinning*), verificação periódica de atualizações em plano de fundo sem bloqueio do usuário, e gestão simplificada via subcomando `navalha config`.

---

## 2. Padrões de Diretórios e Arquivos (XDG)

### 2.1 Configuração do Usuário
- **Caminho Primário:** `$XDG_CONFIG_HOME/navalha/config.toml`
- **Fallback:** `~/.config/navalha/config.toml` (ou `.navalha_config.toml` se não houver HOME)
- **Estrutura TOML (`[update]`):**
  ```toml
  [update]
  channel = "stable"            # "stable" ou "nightly"
  pinned = false                # Trava atualizações automáticas
  pin = ""                      # Versão semântica específica (ex: "0.1.0")
  auto_check = true             # Habilita verificação periódica no startup
  check_interval_hours = 24     # Intervalo mínimo entre checagens remotas
  ```

### 2.2 Estado de Runtime e Cache
- **Caminho Primário:** `$XDG_STATE_HOME/navalha/update_check.json`
- **Fallback:** `~/.local/state/navalha/update_check.json`
- **Estrutura JSON:**
  ```json
  {
    "last_check_timestamp": 1729411200,
    "latest_remote_version": "0.1.1",
    "channel": "stable"
  }
  ```

---

## 3. Endpoints e Resolução Soberana da Banca

- **Canal Stable:**
  - Binário mais recente: `https://banca.2lp.in/downloads/navalha/stable-latest/navalha`
  - Versão mais recente: `https://banca.2lp.in/downloads/navalha/version`
- **Canal Nightly:**
  - Binário mais recente: `https://banca.2lp.in/downloads/navalha/nightly-latest/navalha`
  - Versão mais recente: `https://banca.2lp.in/downloads/navalha/nightly-version`
- **Versão Fixada (Pin):**
  - Binário específico: `https://banca.2lp.in/downloads/navalha/releases/v{pin}/navalha`

---

## 4. Ordem de Precedência de Resolução de Canal

1. Flag de linha de comando: `--channel <stable|nightly>`
2. Variável de ambiente: `NAVALHA_CHANNEL`
3. Configuração persistida: `[update].channel` em `config.toml`
4. Padrão imutável de fallback: `"stable"`

---

## 5. Regras de Negócio e Salvaguardas

1. **Salvaguarda de Pinning:**
   - Se `pinned = true` ou se `pin` estiver preenchido com versão semântica, o comando `navalha update` DEVE recusar a atualização, alertando que a versão está fixada, a menos que a flag `--force` (`-f`) seja informada.
   - O aviso em `stderr` no startup DEVE ser silenciado caso a versão esteja pinada.
2. **Verificação Periódica em Segundo Plano (`run_startup_check`):**
   - No início da execução do CLI, se `auto_check` estiver habilitado e o intervalo `check_interval_hours` tiver expirado, consulta o endpoint remoto de forma leve (timeout curto) e atualiza o arquivo de estado `update_check.json`.
   - Se houver versão mais recente no canal e a versão não estiver pinada, emite aviso discreto em `stderr`.
3. **Substituição Atômica:**
   - O download ocorre em arquivo temporário com permissões `0o755`.
   - A substituição do binário atual é atômica (`fs::rename` ou `persist`), garantindo integridade mesmo em execução ativa.

---

## 6. Subcomandos CLI

- `navalha update [--channel <c>] [--pin <v>] [--force] [--dry-run]`
- `navalha check-update [--channel <c>]`
- `navalha config show` / `navalha config get`
- `navalha config pin [<version>]`
- `navalha config unpin`
- `navalha config set-channel <stable|nightly>`
- `navalha config set-auto-check <true|false>`
