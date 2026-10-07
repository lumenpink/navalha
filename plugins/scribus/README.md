# 📄 Extensão do Navalha para o Scribus (Scripter API)

Script de automação para o **Scribus** (editoração eletrônica aberta e diagramação de publicações livres) que integra o motor gráfico do **Navalha** diretamente ao fluxo de diagramação de fanzines, tabloides e pôsteres.

---

## 🚀 Como Utilizar no Scribus

### 1. Pré-requisito
Certifique-se de que o executável `navalha` está acessível no seu sistema (`$PATH` ou compilado em `~/.cargo/bin/navalha` ou `/usr/local/bin/navalha`).

### 2. Execução Avulsa
1. Abra seu documento no Scribus contendo páginas diagramadas.
2. Crie ou selecione um **Quadro de Imagem** (*Image Frame*) que possua uma fotografia vinculada.
3. No menu superior do Scribus, vá em:
   **Script $\rightarrow$ Executar Script... (Execute Script...)**
4. Navegue até o arquivo `navalha_scribus.py` (ou copie-o para sua pasta de scripts do Scribus).
5. Na janela que se abre:
   - Escolha o **Modo**:
     - `poster`: compõe um pôster brutalista completo na resolução do layout e substitui a foto no quadro.
     - `dither`: aplica apenas o dither/retícula pura mantendo enquadramento.
     - `chapas`: extrai fotolitos separados por cor para **Risografia** e **Silk-Screen**.
   - Selecione a **Estética / Preset** (ex: `viagem-solitaria`, `punk-riot`, `xerox-podre`).
   - Insira uma **Frase de Impacto** opcional para as tarjas.
6. Clique em **Aplicar Filtro**.

---

## 🎨 Fluxo de Matrizes & Chapas para Risografia e Serigrafia

Ao selecionar o modo **`chapas`**:
1. O script gera os arquivos individuais de cada camada de tinta.
2. Automaticamente cria novas **Camadas (Layers)** no Scribus (ex: `Navalha_Chapa_1`, `Navalha_Chapa_2`).
3. Insere e alinha perfeitamente cada matriz em sua respectiva camada com marcas de registro de corte.
4. Você pode ocultar/exibir camadas e exportar PDFs separados prontos para insolamento de telas ou impressão risográfica!

---

## 💻 Execução via Linha de Comando (Standalone)

O script pode ser executado diretamente pelo terminal para testes e pipelines de editoração automatizada:

```bash
python3 plugins/scribus/navalha_scribus.py entrada.jpg -o cartaz.png -p viagem-solitaria -f "MEU CORPO É TRINCHEIRA"
```
