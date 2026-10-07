# 🔌 Plugin do Navalha para o GIMP

Extensão para o **GIMP (GNU Image Manipulation Program)** que integra o motor determinístico do **Navalha** diretamente ao menu de filtros do editor, permitindo aplicar estéticas anti-design, dither e separação de matrizes risográficas diretamente na camada ativa.

---

## 🚀 Como Instalar

### 🐧 Linux
1. Certifique-se de que o executável `navalha` está no seu `$PATH` (ou compile com `cargo build --release -p navalha-cli` e copie para `~/.local/bin/navalha` ou `/usr/local/bin/navalha`).
2. Copie o script `navalha_gimp.py` para o diretório de plug-ins do GIMP:
   - **GIMP 2.10:**
     ```bash
     mkdir -p ~/.config/GIMP/2.10/plug-ins/navalha
     cp plugins/gimp/navalha_gimp.py ~/.config/GIMP/2.10/plug-ins/navalha/
     chmod +x ~/.config/GIMP/2.10/plug-ins/navalha/navalha_gimp.py
     ```
   - **GIMP 3.0:**
     ```bash
     mkdir -p ~/.config/GIMP/3.0/plug-ins/navalha
     cp plugins/gimp/navalha_gimp.py ~/.config/GIMP/3.0/plug-ins/navalha/
     chmod +x ~/.config/GIMP/3.0/plug-ins/navalha/navalha_gimp.py
     ```
3. Reinicie o GIMP.

### 🍎 macOS
1. Certifique-se de que o binário `navalha` está em `/usr/local/bin/navalha` ou no seu PATH.
2. Copie o script para:
   ```bash
   cp plugins/gimp/navalha_gimp.py ~/Library/Application\ Support/GIMP/2.10/plug-ins/
   chmod +x ~/Library/Application\ Support/GIMP/2.10/plug-ins/navalha_gimp.py
   ```

### 🪟 Windows
1. Coloque `navalha.exe` em uma pasta acessível pelo PATH (ex: `C:\Program Files\Navalha\`).
2. Copie `navalha_gimp.py` para:
   `%APPDATA%\GIMP\2.10\plug-ins\`

---

## 🛠️ Como Utilizar no GIMP

1. Abra qualquer imagem ou projeto no GIMP e selecione a camada desejada.
2. Acesse o menu:
   **Filtros (Filters) $\rightarrow$ Artístico (Artistic) $\rightarrow$ Navalha...**
3. Configure os parâmetros na janela modal:
   - **Modo de Operação:** `POSTER` (composição completa em 5 camadas), `DITHER` (difusão de erro crua) ou `CHAPAS` (separação de cor).
   - **Estética / Preset:** selecione um dos 24 presets canônicos (ex: `punk-riot`, `xerox-podre`, `viagem-solitaria`, etc.).
   - **Frase de Impacto:** frase para as tarjas (opcional).
   - **Preservar camada original:** cria uma nova camada com o resultado ou substitui a camada selecionada.
4. Clique em **OK**. O resultado é gerado instantaneamente e integrado ao seu documento.

---

## 💻 Execução via Linha de Comando (Standalone)

O script também pode ser invocado diretamente pelo terminal para automações e scripts em lote:

```bash
python3 plugins/gimp/navalha_gimp.py foto.jpg -o resultado.png -p xerox-podre -f "CORPO É TRINCHEIRA"
```
