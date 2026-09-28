# ⚡🗡️ NAVALHA

> **Motor gráfico e ambiente de composição visual anti-design, fanzine punk e estética de alta fricção.**  
> O avesso das plataformas corporativas pasteurizadas: zero inteligência artificial generativa, matemática de difusão de erro determinística, alto contraste tonal, tipografia crua e privacidade absoluta 100% no cliente.

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Dioxus](https://img.shields.io/badge/dioxus-0.7.10-blue.svg)](https://dioxuslabs.com/)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Norma: GOST](https://img.shields.io/badge/norma-GOST%201.1.0-blueviolet.svg)](.gost/GOST.md)
[![Especificação](https://img.shields.io/badge/spec-SPEC--01-green.svg)](SPEC.md)
[![CI](https://codeberg.org/lumenpink/navalha/workflows/ci.yml/badge.svg)](https://codeberg.org/lumenpink/navalha/actions)
[![Origin](https://img.shields.io/badge/codeberg-navalha-2185d0.svg)](https://codeberg.org/lumenpink/navalha)
[![Online App](https://img.shields.io/badge/online%20app-navalha.2lp.in-ff007f.svg)](https://navalha.2lp.in/)
[![Documentação](https://img.shields.io/badge/docs-navalha.2lp.in%2Fdoc-e2f835.svg)](https://navalha.2lp.in/doc)

---

## ✨ Manifesto e Filosofia

O **Navalha** nasce para quebrar a monotonia visual das interfaces contemporâneas. Não gera imagens "alucinadas" por prompts estocásticos; ele corta, queima níveis, reticula e prensa fotos reais através de algoritmos matemáticos rígidos e determinísticos.

A ideia central é direta:
1. **Entrar uma foto** (ou silhueta com canal alfa).
2. **Entrar uma frase de choque**.
3. **Escolher uma estética anti-design** pré-definida.
4. **Ter um pôster pronto em segundos**, exigindo no máximo micro-edições manuais.

Toda a lógica gráfica é compartilhada simetricamente entre um binário nativo de terminal UNIX (`navalha`) e uma interface web reativa em **Dioxus 0.7.10** compilada para **WebAssembly**, sem backend e sem cookies.

---

## 🎨 Paletas e Afronta Cromática

O sistema rejeita gradientes sutis e trabalha com paletas estritas de 2 a 4 tons puros:

| Paleta | Cores Hexadecimais | Vibe / Referência Cultural |
| :--- | :--- | :--- |
| **Navalha Clássica** | `#000000`, `#FFFFFF`, `#FF0033` | Pôster punk clássico, zine anarquista, xerox de alto contraste |
| **Linóleo & Ouro Queimado** | `#1A1412`, `#D4883A`, `#FFFFFF` | Xilogravura popular, Madame Satã e lâmina no calcanhar |
| **Xerox Podre** | `#1C1917`, `#F5F2EB` | Fotocopiadora velha, papel jornal oxidado, sujeira de toner |
| **Papel Kraft & Olivetti** | `#2B1D0C`, `#C8A870`, `#F5EBE1` | Fichário clandestino dos anos 70, fita de máquina, João W. Nery |
| **Lapa Noturna** | `#0B0B0E`, `#FF007F`, `#00F0FF` | Duotone ácido, rua, cyberpunk sujo, iluminação neon de subúrbio |
| **Teatro Operário** | `#111111`, `#B31B1B`, `#E6E2D8` | Cartaz construtivista teatral, blocos carmesim, Leo Moreira Sá |
| **Risografia Crua** | `#0038A8`, `#FF48B0`, `#FFF200` | Separação física de tintas vibrantes em matriz de impressão |
| **Cyber-BBS Fanzine** | `#001188`, `#FFEE00`, `#FFFFFF` | Estilo BBS dos anos 90, zine digital em azul ultramarino, Amara Moira |
| **Asfalto & Ouro** | `#111111`, `#FFCC00` | Sinalização industrial pesada, perigo urbano, Kátia Tapety |
| **VGA 256** | 256 cores (16 EGA + Cubo 6x6x6 + 24 Tons de Cinza) | Computação dos anos 90, Modo 13h do MS-DOS e demoscene |
| **Brutalismo Concreto** | `#0A0A0A`, `#E5E5E5`, `#FAFAFA` | Concreto aparente, blocos ortogonais maciços, Drica D'arc Meirelles |
| **Trash Polka** | `#111111`, `#7A7A7A`, `#DCDCDC`, `#FFFFFF`, `#BA0C2F` | Realismo fotográfico, difusão 4 níveis e Carmim, Renata Carvalho |
| **Atari 2600 TIA** | 128 cores NTSC Stella | Computação gráfica de 2ª geração, TIA Stella, Jovanna Baby |
| **Mimeógrafo** | `#F4EFE6`, `#432C7A`, `#2A1954` | Papel manilha, violeta metil, anilina e saturação de álcool, Keila Simpson |
| **Xilogravura Sertaneja** | `#D6C4A5`, `#131110` | Papel kraft queimado, preto de fumo, corte anticolonial, Xica Manicongo |
| **Glitch CRT** | `#0D0F12`, `#00FFCC`, `#FF0033`, `#0033FF` | Ruído escuro, fósforo P31, aberração cromática e sinal pirata, Robeyoncé Lima |

Você também pode fornecer qualquer combinação arbitrária de cores via `--paleta "#000,#fff,#ff0033"`.

---

## ⚡ Estéticas Anti-Design Instantâneas (20 Presets Canônicos)

1. * **`punk-riot`**: Contraste brutal, dither de Atkinson, fundo vermelho navalha, frase maciça sangrada no fundo e tarja preta de choque com letras vazadas na base (Claudia Wonder).
2. * **`navalha-sata`**: Xilogravura popular em linóleo, paleta carvão `#1A1412` e ouro cru `#D4883A`, contraste severo e textura cortante (Madame Satã).
3. * **`xerox-podre`**: Simulação de fotocopiadora desgastada com Bayer 8x8, queima severa de meios-tons, tipografia oxidada e faixa com artefatos determinísticos de xerox (Indianarae Siqueira).
4. * **`viagem-solitaria`**: Textura de fichário clandestino dos anos 70, dither Floyd-Steinberg em sépia queimado e tinta carbono Olivetti (João W. Nery).
5. * **`lapa-noturna`**: Duotone magenta e ciano elétrico sobre asfalto escuro, dither Floyd-Steinberg em serpentina e tipografia esticada verticalmente (Linn da Quebrada).
6. * **`teatro-operario`**: Cartaz construtivista com blocos tipográficos maciços em preto chumbo, vermelho carmesim e papel jornal cru em retícula Bayer 4x4 (Leo Moreira Sá).
7. * **`risografia`**: Retícula analógica de ponto de impressão (Halftone 45°) com três camadas vibrantes de tinta (Erika Hilton).
8. * **`fanzine-digital`**: Retícula Bayer 8x8 monocromática em azul ultramarino com tarja amarela no estilo Cyber-BBS dos anos 90 e literatura de guerrilha (Amara Moira).
9. * **`asfalto-alerta`**: Tipografia de advertência industrial em preto e amarelo com marcas de corte e riscos de lâmina (Kátia Tapety).
10. * **`vga256`**: Resolução ultrabaixa tátil (base 320px do Modo 13h), quantização Bayer 4x4, paleta indexada de 256 cores, fundo azul DOS `#0000AA` e tipografia amarelo brilhante `#FFFF55` (Márcia Pantera).
11. * **`cga`**: Paleta fixa de 4 cores do chip Motorola MC6845 (1981, IBM PC 5150) com dither Bayer e preto, ciano, magenta e branco (Neon Cunha).
12. * **`old-mac-pop-art`**: 1-bit Mac 1984 (algoritmo Atkinson Dithering) fundido com serigrafia Pop Art e Andy Warhol no Amiga 1000 de 1985 (Rogéria).
13. * **`ascii-art`**: Conversão por densidade ótica alfanumérica monoespaçada com caracteres ASCII reais (`@%#*+=-:.`) rasterizados sobre a tela, inspirada no pioneiro brasileiro Waldemar Cordeiro (USP 1971) e terminal fósforo verde CRT P1 `#00FF66` (Laerte Coutinho).
14. * **`tabloide-popular`**: Imprensa sensacionalista e rotativa rápida dos anos 80/90 (*Notícias Populares*), unsharp masking agressivo, paleta 4 cores em papel jornal cinza (`#E8E6DF`), caixa amarela de choque e manchete urgente (Luana Muniz).
15. * **`brutalismo`**: Concreto aparente, blocos ortogonais maciços, pré-processamento unsharp 3.5x e binarização estrita por média ponderada de luminância (Drica D'arc Meirelles).
16. * **`trash-polka`**: Colisão entre foto-realismo em 4 níveis de cinza, carmim coagulado `#BA0C2F`, miras circulares com cruz 90° e manchas/splatter procedurais determinísticos (Renata Carvalho).
17. * **`atari-gaming`**: Downsampling assimétrico 2:1 para resolução lógica 160x192, 128 cores oficiais do chip TIA Stella e scanlines CRT pretas intercaladas nas linhas pares (Jovanna Baby).
18. * **`mimeografo`**: Densidade de tinta invertida, Bayer 4x4, sangramento de matriz morfológico de 1px e paleta violeta/anilina sobre papel palha manilha (Keila Simpson).
19. * **`xilogravura`**: Binarização adaptativa local de Sauvola (janela 15x15, k=0.35, R=128) e interferência de fibra direcional de corte de madeira sobre papel kraft queimado (Xica Manicongo).
20. * **`glitch-crt`**: Sync jitter horizontal senoidal com saltos abruptos, aberração cromática R/B desacoplada (+-4px) e barra horizontal rolante de V-Hold com inversão total (Robeyoncé Lima).

> 📖 **Memorial Completo & Teoria:** Leia nosso manifesto histórico aprofundado com as justificativas culturais e técnicas de cada preset em **[`navalha.2lp.in/doc`](https://navalha.2lp.in/doc)** (acessível diretamente pelo cabeçalho na aplicação web).

---

## 🚀 Como Usar

### 1. Linha de Comando (`navalha-cli`)

O binário `navalha` foi construído com ergonomia UNIX: emite logs e deboche no `stderr` e mantém o `stdout` e os arquivos de saída limpos para automações e pipes.

#### 🚀 Geração de Pôster Relâmpago (Foto + Frase + Estética)
```bash
# Pôster punk clássico com um único comando
# Pôster com enquadramento ajustado (zoom-in ou zoom-out), rotação, faixa superior de manchete, marcas de dano e stickers personalizados
navalha poster foto.jpg \
  --frase-superior "MANCHETE DO TOPO" \
  --frase "EU SOBREVIVO AO MUNDO" \
  --estetica punk-riot \
  --rotacao 90 \
  --zoom 1.25 \
  --offset-x 40 \
  --offset-y -20 \
  --dano "tiro:0.30:0.25:1.2","sangue:0.70:0.65:1.5" \
  --sticker "anarchy:0.80:0.20:120:-15","skull:0.20:0.80:90:10" \
  --filtrar-stickers \
  -o poster_final.png

# Exportando diretamente em vetor SVG
navalha poster foto.jpg \
  --frase "A CIDADE É UMA ILUSÃO" \
  --estetica lapa-noturna \
  -o poster.svg
```

> **Enquadramento, Danos e Stickers com Posicionamento Granular na CLI**:
> * `--offset-x` / `--offset-y`: deslocamento horizontal e vertical do sujeito em pixels em relação ao centro (padrão `0`).
> * `--dano`: aceita tanto o nome simples (`tiro`, `sangue`, `pixo`) quanto a sintaxe completa `tipo:posX:posY:escala` (onde `posX` e `posY` são relativos de 0.0 a 1.0 e `escala` é o multiplicador de tamanho, ex: `tiro:0.50:0.40:1.5`).
> * `--sticker`: aceita nome simples (`anarchy`, `skull`) ou a sintaxe completa `tipo:posX:posY:tamanho:rotacao` (onde `tamanho` é em pixels e `rotacao` em graus, ex: `anarchy:0.75:0.25:140:-15`).

#### ⚙️ Dither Direto da Foto
```bash
navalha dither entrada.jpg \
  --resolucao 800 \
  --algoritmo atkinson \
  --paleta classic-punk \
  --escala 2x \
  -o dither.png
```

#### ✂️ Composição Completa com Diagramação em 5 Camadas
```bash
navalha compor entrada.jpg \
  --zoom 0.85 \
  --recorte \
  --recorte-threshold 0.85 \
  --dither atkinson \
  --paleta classic-punk \
  --dano "tiro:0.25:0.25:1.0","faca:0.50:0.70:1.2" \
  --sticker "anarchy:0.85:0.15:110:-10" \
  --texto-fundo "CAOS" \
  --tarja-baixo "NÃO HÁ PAZ SEM RESISTÊNCIA" \
  --distorcao-vertical 1.6 \
  -o composicao.png
```

---

### 2. Interface Web & App (`navalha-web` com Dioxus 0.7.10)

A aplicação web roda **100% no navegador da usuária via WebAssembly**. Suas fotos jamais saem da sua máquina.

* **🌐 Acesso Online Oficial:** [**https://navalha.2lp.in**](https://navalha.2lp.in)
* **📖 Manifesto e Histórico dos Presets:** [**https://navalha.2lp.in/doc**](https://navalha.2lp.in/doc) (memorial visual em tipografia ampliada de alto contraste, com comandos CLI exatos para cada uma das homenageadas e leitura imersiva de todos os presets)

Para executar localmente em sua máquina:
```bash
# Executa o servidor de desenvolvimento local
./scripts/serve.sh
```
Acesse em: `http://localhost:8080`

* **Layout de Tela Única Sem Scroll (100dvh)**: Interface adaptativa que cabe em uma única viewport vertical no desktop, eliminando rolagem externa e organizando controles táteis em três abas especializadas:
  * `⚡ BARRA SUPERIOR`: Cabeçalho integrado com identidade visual, manifesto anti-design e link direto e compacto para a documentação/manifesto (`/doc`), otimizando o espaço vertical útil da aplicação.
  * `💾 EXPORTAÇÃO NO TOPO`: Botões de download fixos no topo do painel de controles com descrições diretas (`💾 BAIXAR PÔSTER (PNG)` e `📐 EXPORTAR VETOR (SVG)`), evitando dúvidas na navegação e mantendo a ação principal sempre visível.
  * `⚡ PÔSTER`: Carregamento, rotação, **posição nos eixos X e Y da foto** (0% a 100% com botão de atalho `↺ CENTRALIZAR`) e **zoom granular da foto** (de 0.25x a 3.0x com botões de atalho 0.7x afastar / 1.5x aproximar; o zoom atua no corte da guilhotina sem alterar o tamanho do ponto do halftone), **faixas de texto superior (opcional) e inferior com consistência visual e estilística automática de acordo com o preset ativo**, e **seletor dropdown brutalista com síntese técnica instantânea**, otimizando o espaço vertical. Todos os ajustes atualizam a imagem instantaneamente em tempo real.
  * `🎨 INTERVENÇÃO`: Danos físicos (tiro, rasgo, chuva, sangue, tinta, faca, pixo, censura, puliça) e adesivos/stickers punk com **subpainéis interativos de ajuste granular de posição (X/Y), tamanho/escala e rotação individual** para cada item ativo, além de opção de paleta cromática e **gerenciador de caixas de texto extras com estilos de acabamento (tarja punk, xerox com falhas de toner e riscos, tape, serrilhado e rasgo) e preview em tempo real**.
  * `⚙️ AJUSTES`: Controles manuais finos de corte guilhotina, nitidez unsharp, distorção tipográfica, algoritmo de dither, paleta cromática e ampliação nearest-neighbor.
* **Experiência Mobile Responsiva Completa (< 900px)**:
  * **Cabeçalho Compacto com Acesso Rápido**: O link para `/doc` é exibido em formato enxuto (`📖 DOCS`) ao lado do logo, sem ocupar linhas adicionais.
  * **Alternador de Visão no Topo**: Alterne instantaneamente entre `[ ⚡ CONTROLES ]` e `[ 👁️ VER CARTAZ ]` em tela cheia sem aperto visual.
  * **Doca Flutuante de Ação Rápida**: Barra inferior ergonômica com botão "💾 BAIXAR PÔSTER (PNG)" e alternância de visualização sempre acessíveis a um toque.
  * **Touch Targets Seguros**: Dimensões ergonômicas (mínimo 44px) e proteção nativa contra zoom indesejado no Safari/iOS.
* **Renderização Assíncrona com Debounce (250ms)**: Digitação fluida e arraste contínuo de controles deslizantes sem congelamento da thread do navegador; o reprocessamento da imagem é executado de modo assíncrono após breve inatividade, mantendo o pôster visível na tela e exibindo um indicador brutalista `[ RENDERIZANDO... ]`.
* **Espelho da CLI em Tempo Real**: O rodapé exibe o comando exato correspondente às escolhas feitas na tela, com botão de cópia rápida.
* **Exportação e Download Direto**: Descarrega o pôster final em PNG de alta resolução com pixels duros e/ou em SVG com os elementos vetoriais.

---

## 📐 Pipeline Gráfico em 5 Estágios

```
[Foto Bruta] 
     ↓
1. Guilhotina (Recorte / Isolação de Sujeito por Luminância e Canal Alfa)
     ↓
2. Preparação de Chapa (Downsampling Forçado, Unsharp Mask Severo e Queima de Níveis)
     ↓
3. Retícula & Dither (Floyd-Steinberg Serpentina, Atkinson, Bayer 4x4/8x8, Halftone Analógico)
     ↓
4. Matriz de Escala (Nearest-Neighbor Obrigatório 1x, 2x, 3x, 4x)
     ↓
5. Diagramação & Prensa (5 Camadas: Fundo, Texto Sangrado, Sujeito, Tarjas Vazadas, Riscos Punk)
```

Consulte o arquivo [`ARCHITECTURE.md`](ARCHITECTURE.md) para todos os detalhes matemáticos e estruturais de cada módulo.

---

## 🛠️ Compilação e Testes

Requisitos: **Rust 2024** (`rustc >= 1.85`), `wasm-pack` e `cargo`.

```bash
# Executar a suíte de testes unitários do workspace
cargo test --workspace

# Compilar o binário release da CLI
cargo build --release -p navalha-cli

# Compilar o pacote Web Wasm
cd crates/navalha-web && wasm-pack build --target web --out-dir dist/pkg

# Ou usar o script unificado
./scripts/build.sh
```

---

## 📜 Licença & Matriz de Conformidade GOST 1.1.0

O projeto **Navalha** adota a política de licenciamento modular por perfil de aplicação definida na norma técnica [GOST v1.1.0](.gost/GOST.md):

* **Código-Fonte da Aplicação e Crates (`crates/navalha-*`):** Licenciado sob a [GNU General Public License v3.0 (GPL-3.0)](LICENSE) (ou qualquer versão posterior).
* **Mídias Visuais, Pôsteres e Imagens de Demonstração (`assets/`, `site/assets/exemplos/`):** Licenciados sob a [Creative Commons Atribuição-CompartilhaIgual 4.0 Internacional (CC-BY-SA 4.0)](LICENSE-MEDIA). A atribuição canônica detalhada de cada fotografia original encontra-se em [docs/assets-attribution.md](docs/assets-attribution.md) e na [documentação oficial](https://navalha.2lp.in/doc).
* **Identidade Visual, Marca e Insígnia do Navalha:** Licenciadas sob [Creative Commons Atribuição-SemDerivações 4.0 Internacional (CC-BY-ND 4.0)](LICENSE-LOGO).
