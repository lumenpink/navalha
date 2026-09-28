# Roadmap: Navalha ⚡🗡️

Planejamento de evolução para as próximas versões do motor e ambiente Navalha.

---

## Versão 0.2.0 (Curto Prazo)
- [ ] **Separação de Chapa para Impressão**:
  - Exportação de arquivos PDF ou PNGs multicamada separados por cor pura (ideal para matrizes de serigrafia, silk-screen e cilindros de risografia).
- [ ] **Mais Presets Culturais Independentes**:
  - `punk-80-br`: Estética crua inspirada no movimento punk do ABC Paulista e fitas K7.
  - `anarco-zine`: Tipografia stencil de guerrilha com marcas de grampo de fanzine.
- [ ] **Distorção Não-Linear de Texto**:
  - Efeito ondulado e rotação individual de palavras em tarjas diagonais.

---

## Versão 0.3.0 (Médio Prazo)
- [ ] **Camada de Colagem Manual Interativa**:
  - Desenho manual de rabiscos, cruzes e setas diretamente sobre o canvas via mouse/touch na interface web.
- [ ] **Importação e Seleção de Fontes Externas**:
  - Suporte ao carregamento de arquivos `.ttf` e `.otf` personalizados diretamente no navegador ou via flag CLI `--fonte`.
- [ ] **Processamento em Lote no Terminal**:
  - Processar diretórios inteiros de imagens com um comando único (`navalha lote input/ -o output/`).

---

## Versão 1.0.0 (Longo Prazo)
- [ ] **Pacote Desktop Híbrido**:
  - Empacotamento desktop nativo multiplataforma (Linux, macOS e Windows) com Dioxus Desktop.
- [ ] **Plugin para Ferramentas de Editoração Aberta**:
  - Extensão para Scribus e GIMP operando como filtro determinístico.
