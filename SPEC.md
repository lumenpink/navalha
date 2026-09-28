# Navalha Technical Specification (SPEC-01)
**Versão da Norma:** `1.0.0`  
**Status:** Oficial / Standard  
**Referência Normativa:** GOST `1.0.0-draft` (Padrão Técnico para Repositórios Abertos)  
**Perfil de Implementação:** `profiles/lumen.md` (Ecossistema Lumen)  
**Autor:** Lumen Pink <hi@lumen.pink>  

---

## 1. Escopo e Filosofia Anti-Design

Esta especificação define a arquitetura técnica, os contratos de API, os formatos de dados, os algoritmos de processamento de imagem e os padrões de governança do **Navalha**.

O **Navalha** é um motor gráfico de alta fricção visual concebido para resgatar a materialidade tátil, caótica e urgente dos fanzines punks, impressões mimeografadas, fotocopiadoras degradadas e arte computacional histórica. Ele rejeita a estética higienizada, asséptica e homogeneizada do design corporativo contemporâneo, fornecendo ferramentas determinísticas para geração instantânea de pôsteres e manipulação brutalista de imagens.

### Princípios Cardinais
1. **Alta Fricção Visual:** Ruído proposital, contraste agressivo, sangria de tinta, retículas mecânicas visíveis, tipografia esticada e sobreposições dissonantes.
2. **Determinismo Bit a Bit Absoluto:** A mesma semente, foto de entrada e parâmetros de configuração **DEVEM** gerar uma saída gráfica binariamente idêntica em qualquer plataforma suportada (CLI nativo Linux x86_64, macOS, Windows ou WebAssembly no navegador).
3. **Zero Dependências Dinâmicas em Runtime:** O núcleo de processamento gráfico (`navalha-core`) opera em memória pura, sem chamadas externas de rede, sem dependência de bibliotecas de sistema operacional (libc dinâmica complexa, fontes instaladas no host ou renderizadores externos) e com a fonte tipográfica brutalista canônica embutida diretamente nos binários (`EMBEDDED_FONT_BYTES`).
4. **Paridade Funcional Total (CLI ↔ Web):** Toda funcionalidade, preset, ajuste de tolerância ou filtro disponível na linha de comando nativa **DEVE** possuir comportamento idêntico na aplicação web baseada em WebAssembly.

---

## 2. Anatomia do Repositório e Conformidade GOST

O repositório do Navalha está estruturado em conformidade estrita com as Seções 2 e 3 da norma GOST:

```
.
├── LICENSE                     # Licença Principal GPL-3.0-or-later
├── LICENSE-MEDIA               # Licença Satélite CC-BY-SA-4.0 (Mídias e Exemplos)
├── LICENSE-LOGO                # Licença Satélite CC-BY-ND-4.0 (Identidade Visual)
├── VERSION                     # Versão semântica pura (ex: 0.1.0-dev.63)
├── CHANGELOG.md                # Histórico de alterações (Keep a Changelog)
├── README.md                   # Documentação executiva, galeria e quickstart
├── SPEC.md                     # Esta especificação técnica normativa
├── .cccprc                     # Configuração de governança e hooks do CCCP
├── Cargo.toml                  # Workspace Rust multi-crate
├── crates/
│   ├── navalha-core/           # Motor determinístico de processamento e composição
│   ├── navalha-cli/            # Interface de linha de comando nativa
│   └── navalha-web/            # Interface reativa em Rust/Dioxus compilada em Wasm
├── site/                       # Portal estático e aplicação web pública
│   ├── index.html              # Ponto de entrada do SPA Wasm (navalha.2lp.in)
│   ├── doc/index.html          # Memorial histórico e especificações dos presets
│   ├── _headers                # Cabeçalhos HTTP de segurança estritos
│   └── assets/                 # Cartazes de referência, ícones e fontes
└── scripts/
    ├── build.sh                # Script de compilação da release nativa e Wasm
    └── serve.sh                # Servidor de desenvolvimento local
```

### 2.1 Licenciamento
Em conformidade com a Seção 2.1 da norma GOST v1.1.0 e as diretrizes do ecossistema Lumen:
- **Código-Fonte e Crates (`crates/navalha-*`):** GNU General Public License v3.0 ([GPL-3.0-or-later](LICENSE)).
- **Mídias Visuais, Pôsteres e Exemplos (`assets/exemplos/`, `site/assets/exemplos/`):** Creative Commons Atribuição-CompartilhaIgual 4.0 Internacional ([CC-BY-SA-4.0](LICENSE-MEDIA)).
- **Identidade Visual, Marca e Insígnia:** Creative Commons Atribuição-SemDerivações 4.0 Internacional ([CC-BY-ND-4.0](LICENSE-LOGO)).

### 2.2 Versionamento Semântico
O versionamento adere ao SemVer 2.0.0 gerenciado automaticamente pela infraestrutura do `cccp`:
- `0.1.0-dev.N`: Fase de desenvolvimento ativo e maturação de especificações.
- Bumps semânticos automáticos: `feat:` incrementa versão menor; `fix:`, `perf:` e `refactor:` incrementam patch; quebras de compatibilidade (`feat!:`) incrementam versão maior.

---

## 3. Arquitetura da Workspace (Multi-Crate Rust)

O projeto é particionado em três crates com limites arquiteturais rigorosos:

### 3.1 `navalha-core` (Motor Gráfico Autônomo)
- **Responsabilidade:** Biblioteca pura em Rust (compatível com `no_std` em camadas algorítmicas, sem chamadas de I/O bloqueante).
- **Módulos Internos:**
  - `prep`: Orientação, rotação em ângulos retos, zoom granular, cálculo de limiar Otsu e preparação de chapa (*downsampling*, *unsharp masking*, correção de níveis e gama).
  - `guillotine`: Segmentação e recorte por limiar de luminância, com inversão e preenchimento de fundo opcional.
  - `dither`: Algoritmos de reticulagem analógica e difusão de erro.
  - `ascii`: Conversão alfanumérica por densidade óptica, rampa de caracteres de Waldemar Cordeiro (USP 1971) e rasterização tipográfica monoespaçada.
  - `palette`: Mapeamento de espaço de cores, cálculo de distância euclidiana perceptual e catálogo de paletas históricas (CGA, VGA, Pop Art, Trash Polka, Fósforo Verde, Tabloide).
  - `scale`: Redimensionamento determinístico por vizinho mais próximo (*Nearest-Neighbor*).
  - `press`: Motor de composição de pôster em camadas, tipografia vazada/em tarja, efeitos de intervenção física (danos, tiros, rasgos, sangue) e colagem de stickers.
  - `presets`: Definição declarativa dos presets anti-design e tradução paramétrica para o pipeline.
  - `svg`: Serializador vetorial SVG determinístico para exportação gráfica em alta resolução.

### 3.2 `navalha-cli` (Linha de Comando Nativa)
- **Responsabilidade:** Binário executável independente compilado nativamente para automação, pipelines de terminal e renderização em lote.
- **Tecnologias:** `clap` (v4, derive), `image`, `navalha-core`.
- **E/S:** Suporte a arquivos locais no disco e suporte a streaming via `stdin` (`-`) e `stdout`.

### 3.3 `navalha-web` (Aplicação Interativa Wasm)
- **Responsabilidade:** Single Page Application (SPA) reativa cliente que executa o motor completo localmente no navegador via WebAssembly sem depender de nenhum processamento de servidor.
- **Tecnologias:** `dioxus` (v0.7, web), `wasm-bindgen`, `web-sys`, `base64`.
- **Cancelamento Cooperativo & Cache:** Interrupção imediata de pipelines de renderização em andamento ao registrar novos inputs (`process_subject_cancellable` e `render_poster_cancellable`), combinada a cache de chapa de sujeito (`SubjectLayerCache`) para garantir 60fps na manipulação de camadas secundárias.

---

## 4. O Pipeline Gráfico dos 5 Estágios

O processamento do sujeito fotográfico segue um pipeline linear rigoroso de 5 estágios determinísticos:

```
[Entrada Bruta]
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ Estágio 0: Enquadramento & Zoom Granular               │
│ - Rotação retangular: 0°, 90°, 180°, 270°              │
│ - Zoom proporcional contínuo: 0.1x a 5.0x              │
└────────────────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ Estágio 1: Guilhotina de Silhueta (Guillotine)         │
│ - Limiar manual (0.0 a 1.0) ou Otsu automático         │
│ - Inversão de máscara e fundo transparente/sólido      │
└────────────────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ Estágio 2: Preparação de Chapa (Prep Plate)           │
│ - Downsampling na resolução de trabalho (320px–720px)  │
│ - Unsharp Masking agressivo (acentuação de bordas)     │
│ - Levels: corte de sombras, altas luzes e curva gama   │
└────────────────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ Estágio 3: Retícula & Dither Analógico (Dither)        │
│ - Floyd-Steinberg (serpentina alternada)               │
│ - Atkinson (6 vizinhos, dispersão 75%, 1/8)            │
│ - Bayer 4x4 e Bayer 8x8 (dither ordenado matricial)   │
│ - Halftone mecânico (pontos em 45°, 0° ou 90°)         │
│ - ASCII Art Tipográfico Real (rampa Cordeiro 1971)     │
└────────────────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ Estágio 4: Matriz de Escala (Nearest-Neighbor)         │
│ - Escala 1x, 2x, 3x, 4x sem interpolação linear        │
│ - Preservação dos grãos e pontos de retícula           │
└────────────────────────────────────────────────────────┘
       │
       ▼
[Chapa Processada para Composição]
```

### 4.1 Especificação Detalhada dos Algoritmos de Dither

#### 4.1.1 Floyd-Steinberg em Serpentina
Implementa a matriz clássica com dispersão de erro unidirecional corrigida por varredura alternada em serpentina (linhas pares da esquerda para direita; linhas ímpares da direita para esquerda) para eliminar artefatos visuais direcionais (*worming*):
- Direita: $7/16$
- Esquerda-baixo: $3/16$
- Centro-baixo: $5/16$
- Direita-baixo: $1/16$

#### 4.1.2 Atkinson Dithering (Macintosh 1984)
Diferente dos algoritmos tradicionais de conservação total de energia, o algoritmo de Bill Atkinson distribui apenas $3/4$ ($75\%$) do erro acumulado para 6 vizinhos ($1/8$ para cada), descartando o quarto restante:
- Direita (+1, 0): $1/8$
- Direita dupla (+2, 0): $1/8$
- Esquerda-baixo (-1, +1): $1/8$
- Centro-baixo (0, +1): $1/8$
- Direita-baixo (+1, +1): $1/8$
- Centro-baixo duplo (0, +2): $1/8$
*Efeito Estético:* Preserva brancos limpos e sombras profundas, gerando a textura inconfundível dos primeiros monitores monocromáticos da Apple.

#### 4.1.3 Bayer Ordenado (4x4 e 8x8)
Aplica matrizes de limiar fixo baseadas em recursão bit-reversal para simular a quantização gráfica rígida dos primeiros microcomputadores pessoais e cartões de vídeo CGA/VGA.

#### 4.1.4 Halftone Analógico
Simula a retícula física de impressão tipográfica e rotativa offset. A imagem é particionada em células de dimensão $S \times S$ rotacionadas no ângulo $\theta$ (tipicamente $45^\circ$). A densidade de tinta média da célula dita o raio de um círculo sólido central, reproduzindo o clássico ponto de impressão.

#### 4.1.5 ASCII Art Tipográfico Real (Waldemar Cordeiro 1971)
Homenagem à obra *“Derivadas de uma Imagem”* criada no computador IBM 360/44 do Centro de Computação da USP:
- A chapa do sujeito é decomposta em uma grade alfanumérica monoespaçada de células $9 \times 12$ px ($\sim 82$ colunas).
- A luminância BT.601 de cada célula é mapeada na rampa canônica de densidade óptica:
  $$\text{RAMP} = \text{“ .:-=+*\#\%@”}$$
- Glifos vetoriais reais da fonte embutida são rasterizados com antialiasing subpixel sobre fundo escuro CRT (`#050A05`), colorizados de acordo com a gradação em 3 níveis de fósforo da paleta `terminal_green` (`#005514`, `#00AA32`, `#00FF66`).

---

## 5. Motor de Composição e Imprensa (`render_poster`)

O cartaz final é composto deterministamente em dimensões padronizadas de **$800 \times 1100$ pixels** através de 6 camadas sequenciais:

1. **Camada 0 — Fundo Sólido Chapado:** Preenchimento integral com a cor primária de fundo da estética (ex: Vermelho Navalha `#FF0033`, Preto CRT `#050A05`, Asfalto `#0B0B0E` ou Papel Jornal `#E8E6DF`).
2. **Camada 1 — Tipografia de Fundo Sangrada:** Letras maciças (tamanho até 96pt) com distorção vertical e horizontal agressiva, posicionadas no fundo da cena como marca d'água de choque.
3. **Camada 2 — Chapa do Sujeito:** A imagem processada nos 5 estágios é blendada sobre a cena com suporte a centralização automática e deslocamento customizado nos eixos horizontal e vertical (`subject_offset_x` e `subject_offset_y`), respeitando a máscara de alfa da guilhotina. O buffer processado é mantido em cache reativo (`SubjectLayerCache`), permitindo re-renderização em milissegundos quando apenas camadas secundárias (textos, faixas, stickers ou danos) forem manipuladas.
4. **Camada 3 — Textos de Intervenção em Tarja:** Frase principal, faixa superior e caixas de texto complementares com suporte a múltiplas linhas (`\n`), ajuste vertical dinâmico da caixa de tarja, modo adaptável ao preset ativo (`default_banner_element` com sincronização automática de estilo de corte, família de fonte e cores), seleção de família tipográfica (`ImpactBrutal`, `MonoArchive`, `SerifEditorial`), tamanho de fonte configurável com modo auto-fit inteligente, livre reposicionamento nos eixos horizontal e vertical (`--banner-x`, `--banner-y`), inclinação configurável (`--banner-rot`), seleção de cor do texto (`--banner-fg`), fundo sólido ou transparente (`--banner-bg`), estilo de borda (`--banner-borda`), espessura (`--banner-borda-largura`), cor da borda (`--banner-borda-cor`) e efeito de acabamento sólido ou degradê/desgaste xerox (`--banner-efeito`).
5. **Camada 4 — Elementos de Intervenção e Marcas de Lâmina:**
   - Fio perimetral de corte com marcas de registro em cruz nos quatro cantos.
   - Riscos vetoriais de estilete/lâmina simulando intervenção física sobre a chapa.
   - Efeitos de agressão física: `tiro`, `rasgo`, `chuva`, `sangue`, `tinta`, `faca`, `pixo`, `censura`, `pulica`.
6. **Camada 5 — Stickers Temáticos e Elementos de Recorte:** Selos de protesto gráficos (anarquia, caveira, lâmina, biohazard, smiley, etc.) e suporte a upload de imagens personalizadas do usuário (`custom_image` ou `--sticker-custom`) convertidas em elementos de colagem com redimensionamento proporcional inteligente, colados sobre o cartaz com suporte a rotação angular contínua, filtragem opcional de cor com a paleta do preset, acabamento de corte/borda personalizável (`StickerBorder`: die-cut vinílico com vinco externo, picote perfurado com espaçamento de destaque, borda serrilhada em dente-de-serra, linha preta ou sem borda), espessura ajustável em pixels e renderização vetorial SVG com embutimento de imagem em Base64.

---

## 6. Catálogo Canônico de 20 Presets Anti-Design (Ordem Intercalada)

| # | Slug | Nome da Estética / Homenagem | Resolução Base | Algoritmo de Dither | Paleta Tonal | Filosofia Gráfica |
|---|---|---|:---:|:---:|:---:|---|
| 01 | **`punk-riot`** | Punk Riot / Claudia Wonder | 720 px | Atkinson | Clássica Punk (Preto, Branco, Vermelho) | Alto contraste, fanzine impresso em mimeógrafo, tarja de choque preta na base. |
| 02 | **`navalha-sata`** | Linóleo & Ouro Queimado / Madame Satã | 720 px | Atkinson | Carvão `#1A1412`, Ouro `#D4883A`, Branco | Xilogravura popular em linóleo, navalha no calcanhar, mito da Lapa carioca. |
| 03 | **`xerox-podre`** | Xerox Podre / Indianarae Siqueira | 640 px | Bayer 8x8 | Toner Oxidado e Papel Jornal | Simulação de fotocopiadora desgastada, queima de meios-tons e acúmulo de fuligem. |
| 04 | **`viagem-solitaria`** | Papel Kraft & Olivetti / João W. Nery | 640 px | Floyd-Steinberg | Sépia `#2B1D0C`, Ocre Kraft `#C8A870`, Branco | Fichário clandestino dos anos 70, fita de máquina de escrever e ativismo transmasculino. |
| 05 | **`lapa-noturna`** | Lapa Noturna / Linn da Quebrada | 720 px | Floyd-Steinberg | Duotone Ciano e Magenta Elétrico | Boemia urbana, tipografia esticada verticalmente sobre fundo asfalto escuro. |
| 06 | **`teatro-operario`** | Teatro Operário / Leo Moreira Sá | 720 px | Bayer 4x4 | Preto Chumbo `#111111`, Carmesim `#B31B1B`, Papel | Cartaz construtivista com blocos tipográficos maciços e dramaturgia marginal. |
| 07 | **`risografia`** | Risografia Crua / Erika Hilton | 720 px | Halftone ($45^\circ$, 6px) | Rosa Flúor, Azul Royal, Amarelo Riso | Ponto de retícula mecânica nítido simulando impressão analógica de cilindro. |
| 08 | **`fanzine-digital`** | Fanzine Digital / Amara Moira | 640 px | Bayer 8x8 | Azul Ultramarino `#001188`, Amarelo `#FFEE00` | Cultura fanzine dos anos 90, BBS, literatura de guerrilha e desobediência acadêmica. |
| 09 | **`asfalto-alerta`** | Asfalto & Ouro / Kátia Tapety | 720 px | Atkinson | Amarelo Industrial e Preto Piche | Sinalização tática de trânsito, riscos de lâmina cruzados e perigo iminente. |
| 10 | **`vga256`** | VGA 256 (13h) / Márcia Pantera | 320 px | Bayer 4x4 | Paleta Indexada 256 Cores | Visual DOS autêntico dos anos 90, fundo azul `#0000AA` e texto amarelo neon `#FFFF55`. |
| 11 | **`cga`** | CGA (1981) / Neon Cunha | 320 px | Bayer 4x4 | 4 Cores: Preto, Ciano, Magenta, Branco | O hardware gráfico original do IBM PC 5150 em resolução 320x200 de 2 bits e estética afropunk. |
| 12 | **`old-mac-pop-art`** | Old Mac Pop Art / Rogéria | 512 px | Atkinson | 5 Cores: Preto, Branco, Rosa, Amarelo, Turquesa | Encontro entre a serigrafia Pop Art e a difusão clássica do Mac 128K de 1984. |
| 13 | **`ascii-art`** | ASCII Art (1971) / Laerte Coutinho | $\sim 82$ cols | Raster ASCII Real | Fósforo Verde CRT P1 (4 tons de verde) | Caracteres alfanuméricos tipográficos reais (`@%#*+=-:.`) sobre tela de fósforo. |
| 14 | **`tabloide-popular`** | Tabloide Popular / Luana Muniz | 720 px | Floyd-Steinberg | Preto Chumbo, Vermelho, Amarelo, Papel Jornal | Imprensa sensacionalista dos anos 80/90 com manchete urgente e unsharp agressivo. |
| 15 | **`brutalismo`** | Brutalismo Concreto / Drica D'arc Meirelles | 720 px | Threshold Global | Fundo Concreto `#E5E5E5`, Preto Piche `#0A0A0A`, Gesso | Blocos ortogonais maciços, unsharp brutal 3.5x e solidez intransigente de acolhimento. |
| 16 | **`trash-polka`** | Trash Polka / Renata Carvalho | 720 px | Floyd-Steinberg 4 Níveis | Preto, Cinzas, Branco, Carmim `#BA0C2F` | Realismo fotográfico, miras circulares, escala milimetrada e manchas procedurais determinísticas. |
| 17 | **`atari-gaming`** | Atari Gaming / Jovanna Baby | 160 px | AtariGaming TIA | 128 Cores Stella NTSC | Downsampling assimétrico 2:1, tabela de 128 cores do chip TIA e scanlines CRT pares pretas. |
| 18 | **`mimeografo`** | Mimeógrafo / Keila Simpson | 640 px | Bayer 4x4 + Bleed | Manilha `#F4EFE6`, Violeta `#432C7A`, Álcool `#2A1954` | Densidade invertida de tinta, sangramento morfológico 1px e boletins comunitários de resistência. |
| 19 | **`xilogravura`** | Xilogravura / Xica Manicongo | 720 px | Sauvola Woodcut | Kraft `#D6C4A5`, Preto Fumo `#131110` | Binarização adaptativa de Sauvola ($15 \times 15$, $k=0.35$, $R=128$) e estrias direcionais de madeira. |
| 20 | **`glitch-crt`** | Glitch CRT / Robeyoncé Lima | 640 px | GlitchCrt Jitter | Ruído `#0D0F12`, Fósforo `#00FFCC`, Vermelho, Azul | Sync jitter senoidal $\Delta x(y)$, split cromático R/B e barra horizontal de V-Hold a 75% da altura. |

### 6.1 Estilização e Adequação Canônica de Faixas por Preset

Cada preset anti-design define de forma canônica sua tipografia, acabamento e contraste de tarja através de `AestheticPreset::default_banner_element`:
- **`punk-riot`**: Impact Brutal, borda sólida de 2px vermelha punk (`#FF0033`), fundo preto e letras brancas vazadas.
- **`navalha-sata`**: Serif Editorial, borda sólida de 2px ouro cru (`#D4883A`), fundo carvão linóleo (`#1A1412`) e letras brancas.
- **`xerox-podre`**: Impact Brutal, acabamento Xerox com micro-cavidades e perda de toner, borda picotada de 3px (`#322D28`), fundo toner escuro e letras oxidado-vazadas.
- **`viagem-solitaria`**: Mono Archive, borda tracejada de 2px kraft (`#C8A870`), fundo marrom quente Olivetti (`#2B1D0C`) e letras bege papel.
- **`lapa-noturna`**: Impact Brutal, borda sólida de 2px magenta elétrico (`#FF0080`), fundo noite profunda (`#0A0A0E`) e letras ciano neon.
- **`teatro-operario`**: Impact Brutal largo, borda sólida pesada de 4px vermelho operário (`#B31B1B`), fundo carvão construtivista e letras bege jornal.
- **`risografia`**: Sans Moderna, borda em picote mecânico de 2px azul royal (`#0055FF`), fundo rosa fluorescente (`#FF48B0`) e letras amarelo solar.
- **`fanzine-digital`**: Mono Archive, borda tracejada de 2px azul ultramarino BBS (`#001188`), fundo amarelo CRT (`#FFEE00`) e texto azul escuro.
- **`asfalto-alerta`**: Impact Brutal pesado, borda sólida de 4px preto piche (`#111111`), fundo amarelo sinalização tática (`#FFCC00`) e texto preto vazado.
- **`vga256`**: Mono Archive, borda tracejada de 2px azul DOS (`#0000AA`), fundo preto maciço e texto amarelo neon Modo 13h (`#FFFF55`).
- **`cga`**: Mono Archive, borda sólida de 2px magenta CGA (`#FF55FF`), fundo preto e texto ciano brilhante IBM 5150 (`#55FFFF`).
- **`old-mac-pop-art`**: Sans Moderna, borda sólida de 3px rosa pop art (`#FF69B4`), fundo branco estampa e texto preto alto contraste.
- **`ascii-art`**: Mono Archive, borda tracejada de 2px verde fósforo (`#00FF66`), fundo preto terminal CRT e texto verde brilhante.
- **`tabloide-popular`**: Impact Brutal rotativa, borda sólida de 3px amarelo manchete (`#FFCC00`), fundo chumbo escuro e texto branco de alto impacto.
- **`brutalismo`**: Impact Brutal condensado caixa-alta, borda sólida pesada de 4px preto piche (`#0A0A0A`), fundo preto maciço e texto branco gesso (`#FAFAFA`).
- **`trash-polka`**: Mono Archive máquina mecânica, borda tracejada de 2px preto carvão (`#111111`), fundo carmim (`#BA0C2F`) e texto branco cirúrgico.
- **`atari-gaming`**: Mono Archive console anos 80, borda sólida de 3px carmim TIA (`#D06C6C`), fundo preto absoluto e texto amarelo Stella (`#FCFC70`).
- **`mimeografo`**: Mono Archive estêncil a álcool, acabamento xerox/tinta com borda em picote de 2px saturação escura (`#2A1954`), fundo violeta metil (`#432C7A`) e texto manilha (`#F4EFE6`).
- **`xilogravura`**: Impact Brutal talhado em madeira, borda sólida de 4px kraft queimado (`#D6C4A5`), fundo preto de fumo (`#131110`) e texto kraft.
- **`glitch-crt`**: Mono Archive terminal quebrado, borda tracejada de 2px vermelho aberração (`#FF0033`), fundo ruído escuro (`#0D0F12`) e texto fósforo verde/ciano (`#00FFCC`).

---

## 7. Interface de Linha de Comando (CLI)

O binário `navalha` disponibiliza subcomandos para operação relâmpago e composição cirúrgica:

### 7.1 Subcomando `navalha poster`
```bash
navalha poster <INPUT> [OPTIONS]
```
- `-f, --frase <TEXTO>`: Frase de impacto do pôster (padrão: `"EU SOBREVIVO AO MUNDO"`).
- `--frase-superior <TEXTO>`: Manchete opcional da faixa superior.
- `-e, --estetica <SLUG>`: Estética anti-design escolhida (padrão: `punk-riot`).
- `--rotacao <GRAUS>`: Rotação prévia da foto (0, 90, 180, 270).
- `--zoom <FATOR>`: Zoom granular (ex: 0.5 para afastar, 1.8 para aproximar).
- `--offset-x <PIXELS>`: Deslocamento horizontal do sujeito em pixels (padrão: 0 = centralizado).
- `--offset-y <PIXELS>`: Deslocamento vertical do sujeito em pixels (padrão: 0 = centralizado).
- `--banner-x <FATOR>`: Posição horizontal da faixa principal (0.0 a 1.0).
- `--banner-y <FATOR>`: Posição vertical da faixa principal (0.0 a 1.0).
- `--banner-rot <GRAUS>`: Ângulo de inclinação da faixa principal (-180° a 180°).
- `--banner-fg <COR>`: Cor do texto da faixa principal (`white`, `black`, `red`, `#hex`).
- `--banner-bg <COR>`: Cor de fundo da faixa principal (`black`, `red`, `none` para vazado).
- `--banner-borda <ESTILO>`: Estilo de borda (`solida`, `tracejada`, `picote`, `durex`).
- `--banner-borda-largura <PX>`: Espessura da borda em pixels.
- `--banner-borda-cor <COR>`: Cor customizada da borda da faixa.
- `--banner-efeito <EFEITO>`: Acabamento visual (`solida`, `xerox`).
- `--sem-faixa`: Desativa e remove completamente as faixas decorativas de texto.
- `--sem-recorte`: Desativa a guilhotina, mantendo a foto original sem corte de silhueta.
- `--recorte <BOOL>`: Força ativação (`true`) ou desativação (`false`) da guilhotina.
- `--recorte-threshold <FATOR>`: Limiar manual de luminância do recorte (0.0 a 1.0).
- `--recorte-inverter`: Inverte a polaridade da máscara de silhueta da guilhotina.
- `--dano <LISTA>`: Danos físicos (ex: `tiro,sangue` ou `"tiro:0.5:0.4:1.5"`).
- `--sticker <LISTA>`: Stickers gráficos (ex: `anarchy,skull` ou sintaxe posicional estendida `"skull:0.5:0.5:100:15:diecut:4"`).
- `--sticker-custom <ESPEC>`: Stickers/recortes com imagens customizadas do usuário (`"CAMINHO[:posX:posY[:tamanho[:rotacao[:borda[:largura]]]]]"`).
- `--sticker-borda <ESTILO>`: Estilo padrão de recorte e borda para stickers (`diecut`, `picote`, `serrilhada`, `preta`, `nenhuma`).
- `--sticker-borda-largura <PX>`: Espessura da borda/recorte em pixels (padrão: 4).
- `--filtrar-stickers`: Harmoniza os stickers com a paleta de cores do preset.
- `--texto-extra <ESPEC>`: Adiciona caixas de texto complementares (`"TEXTO:x:y:tam:rot:acabamento"`).
- `-o, --saida <ARQUIVO>`: Arquivo de saída PNG ou SVG (padrão: `poster.png`).

### 7.2 Subcomando `navalha ascii`
```bash
navalha ascii <INPUT> [--colunas <N>] [--inverter]
```
Converte qualquer imagem em texto puro monoespaçado e despeja diretamente no `stdout` do terminal Linux, sem interface gráfica.

---

## 8. Rigor de Engenharia, Qualidade e Governança GOST

### 8.1 Política Zero Warnings
A compilação de todo o workspace é executada com tolerância zero a alertas:
```bash
RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features
```
Qualquer warning gerado por código não utilizado, dead code ou convenções de estilo quebra imediatamente o pipeline de integração contínua.

### 8.2 Rastreabilidade e Commits Atômicos (`cccp`)
Em cumprimento à Seção 5 da norma GOST:
- **Zero Commits sem Ticket:** Nenhum commit é realizado sem associação explícita ao número da issue correspondente (`#<id>`).
- **Ciclo Paired Commit:** Toda alteração com impacto em comportamento ou interfaces é dividida em dois passos atômicos consecutivos:
  1. *Commit de Implementação:* Código-fonte, testes e assets técnicos (`cccp commit "<type>(<scope>): <msg> (#<id>) closes #<id>"`).
  2. *Commit de Documentação:* Manuais, `README.md`, `site/doc` e especificações (`cccp commit "docs(<scope>): <msg> (#<id>)"`).
- Todas as operações de commit, linting e push são executadas exclusivamente via `cccp` (`cccp commit`, `cccp lint`, `cccp push`).

### 8.3 Banners de Metadados Sociais (Padrão KokoroSim / GOST Seção 11)
O banner oficial de compartilhamento para redes sociais (`site/assets/og_preview.png`) adere às regras anti-recorte:
- Dimensão: **$1200 \times 630$ pixels** (formato PNG RGB).
- Safe Zone Central 4:3: Todo o conteúdo textual, título e selos gráficos estão estritamente contidos no retângulo central de **$720 \times 500$ pixels**, com margens livres para corte lateral em clientes móveis (WhatsApp/Telegram).
- Título conciso (máximo 45 caracteres) e descrição auto-contida (120–155 caracteres) declarados nas meta tags Open Graph e Twitter Cards.

---

## 9. Memorial Histórico e Acessibilidade

O projeto mantém em seu memorial público ([`navalha.2lp.in/doc`](https://navalha.2lp.in/doc)) a documentação aprofundada da biografia, contexto político e legado dos ícones homenageados em cada estética gráfica:

1. **Claudia Wonder** (Punk Riot) — Pioneira do punk rock travesti, performer e cronista da noite underground paulistana.
2. **Madame Satã** (Navalha Satã) — Malandragem clássica, navalha no calcanhar, mito da Lapa e insubmissão à repressão policial.
3. **Indianarae Siqueira** (Xerox Podre) — Ativista histórica, fundadora da Casa Nem e líder da ocupação política dos corpos dissidentes.
4. **João W. Nery** (Viagem Solitária) — Primeira autobiografia de homem trans no Brasil (*Viagem Solitária*), escritor e psicólogo pioneiro.
5. **Linn da Quebrada** (Lapa Noturna) — Pajubá, afrofuturismo, funk experimental e revolução na música e na TV (*Pajubá*, *Traveqa*).
6. **Leo Moreira Sá** (Teatro Operário) — Ator e dramaturgo fundador da cena teatral transmasculina de arena e resistência política.
7. **Erika Hilton** (Risografia Crua) — Uma das primeiras deputadas federais trans da história do Congresso Nacional e liderança legislativa.
8. **Amara Moira** (Fanzine Digital) — Doutora em Teoria Literária (UNICAMP), ensaísta, escritora e fratura da barreira acadêmica em defesa das profissionais do sexo.
9. **Kátia Tapety** (Asfalto Alerta) — Primeira travesti eleita para um cargo político no Brasil (Piauí, 1992), pioneira da luta rural e comunitária.
10. **Márcia Pantera** (VGA 256) — Ícone das pistas e criadora do movimento artístico do Bate-Cabelo.
11. **Neon Cunha** (CGA) — Mulher negra trans, ativista independente, marco na despatologização da transgeneridade e retificação de registro sem laudo médico.
12. **Rogéria** (Old Mac Pop Art) — Vedete e atriz internacional, autointitulada com imenso afeto "a travesti da família brasileira".
13. **Laerte Coutinho** (ASCII Art) — Mestra do quadrinho nacional, cartunista e cofundadora da revista *Chiclete com Banana*.
14. **Luana Muniz** (Tabloide Popular) — "A Rainha da Lapa", líder comunitária e autora da frase imortal: *"Travesti não é bagunça!"*.
15. **Drica D'arc Meirelles** (Brutalismo) — Fundadora e coordenadora da ADEH em Santa Catarina, combatente da letalidade policial e acolhimento de base.
16. **Renata Carvalho** (Trash Polka) — Atriz, dramaturga santista, formuladora da *Transpofagia* e protagonista de *O Evangelho Segundo Jesus, Rainha do Céu*.
17. **Jovanna Baby** (Atari Gaming) — Fundadora da ASTRAL (1992) e idealizadora do I Encontro Nacional de Travestis e Liberados (ENTLAIDS, 1993).
18. **Keila Simpson** (Mimeógrafo) — "O Anjo do Bixiga", fundadora do Palacete dos Padres (1986), primeiro abrigo da América Latina para travestis e pessoas com HIV/Aids.
19. **Xica Manicongo** (Xilogravura) — Primeira travesti não-indígena documentada no Brasil colonial (Salvador, séc. XVI), processada pelo Santo Ofício da Inquisição.
20. **Robeyoncé Lima** (Glitch CRT) — Primeira travesti com carteira da OAB no Brasil, fundadora da ATRAC e formuladora de diretrizes de saúde no Ministério da Saúde.
