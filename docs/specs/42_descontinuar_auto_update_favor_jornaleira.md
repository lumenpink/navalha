# Especificação Técnica: Descontinuação do Auto-Update Interno em Favor da Jornaleira

**Ticket:** #42  
**Componente:** `crates/navalha-cli`  
**Status:** Aprovado para Implementação  
**Padrão:** Norma GOST 2.0 & Arquitetura Jornaleira  

---

## 1. Contexto e Motivação

O aplicativo `navalha-cli` executava verificação autônoma de novas versões no momento da inicialização (`run_startup_check` em `main.rs`), além de subcomandos de self-update com substituição em tempo de execução de binários locais.

Com a consolidação do ecossistema e do gestor de pacotes **Jornaleira**, a gestão do ciclo de vida, distribuição e atualizações de todas as ferramentas de linha de comando passa a ser centralizada. A execução de requisições de rede ou mutações de binários no arranque viola o princípio de ferramentas determinísticas, 100% offline por padrão e introduz latência desnecessária no startup.

---

## 2. Requisitos e Contratos

### 2.1 Inicialização Limpa e 100% Offline
- Remover a invocação de `run_startup_check` da rotina `main()` em `crates/navalha-cli/src/main.rs`.
- A função `run_startup_check` em `crates/navalha-cli/src/update.rs` deve operar como no-op seguro, sem disparar threads de rede nem ler ou escrever arquivos de estado em disco durante a execução normal.

### 2.2 Orientação nos Subcomandos de Update
- Os subcomandos `navalha update` e `navalha check-update` devem emitir mensagem instrutiva de governança centralizada pela Jornaleira:
  - `"★ GOSPLAN: A distribuição de binários e atualizações é governada centralmente pela Jornaleira ('jornaleira upgrade navalha')."`
- Não realizar download nem substituição de binários em background sem supervisão.

### 2.3 Preservação das Configurações XDG
- Manter o suporte e integridade das funções de configuração XDG (`navalha config`), preservando schema e leitura para que a Jornaleira ou o usuário possam consultar ou definir canal e versões fixadas (*pinning*).
