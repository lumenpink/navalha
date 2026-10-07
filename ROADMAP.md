# Roadmap: Navalha ⚡🗡️

Planejamento de evolução para as próximas versões do motor e ambiente Navalha.

---

## Versão 0.2.0 (Entregue)
- [x] **Separação de Chapa para Impressão** (`#3`):
  - Exportação de arquivos PDF ou PNGs multicamada separados por cor pura (ideal para matrizes de serigrafia, silk-screen e cilindros de risografia).
- [x] **Mais Presets Culturais Independentes** (`#4`, `#5`, `#14`):
  - `punk-80-br`: Estética crua inspirada no movimento punk do ABC Paulista e fitas K7.
  - `anarco-zine`: Tipografia stencil de guerrilha com marcas de grampo de fanzine.
  - `paulete-furacao` e `symmy-larrat`: Comunicação popular e imprensa amazônica viva.
- [x] **Distorção Não-Linear de Texto** (`#6`):
  - Efeito ondulado e rotação individual de palavras em tarjas diagonais determinísticas.

---

## Versão 0.3.0 (Entregue)
- [x] **Camada de Colagem Manual Interativa** (`#7`):
  - Desenho manual de rabiscos, cruzes e setas diretamente sobre o canvas via mouse/touch na interface web com espelhamento SVG.
- [x] **Tipografia Determinística & Fontes Livres Embutidas** (`#8`, `#20`):
  - Inclusão embutida de *Special Elite* (typewriter vintage) e *DejaVu Serif* (editorial condensada), com suporte a modo caótico *Ransom Note* e flag CLI `--banner-fonte`.
- [x] **Processamento em Lote no Terminal** (`#9`):
  - Processamento de diretórios inteiros de imagens com paralelismo nativo (`navalha lote input/ -o output/`).

---

## Versão 1.0.0 (Em Desenvolvimento)
- [ ] **Plugin para Ferramentas de Editoração Aberta** (`#11`):
  - Extensão para Scribus e GIMP operando como filtro determinístico direto nas ferramentas livres de design gráfico.
- [x] ~~**Pacote Desktop Híbrido**~~ (`#10` - Descontinuado):
  - Substituído com paridade total pelo binário nativo UNIX `navalha-cli` e pela aplicação web WebAssembly com suporte offline e instalação como PWA (evitando dependências pesadas de WebKitGTK e mantendo 100% de privacidade).
