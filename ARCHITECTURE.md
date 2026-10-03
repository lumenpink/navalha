# Arquitetura Técnica: Navalha ⚡🗡️

> Documento de referência de engenharia, pipeline gráfico e determinismo do motor **Navalha**.

---

## 1. Topologia do Workspace (Rust 2024)

O projeto é organizado como um **Cargo Workspace** modularizado para maximizar o compartilhamento de código e manter a simetria entre o terminal nativo e a WebAssembly:

```
crates/
├── navalha-core/    # Motor gráfico puro em Rust 2024 (agnóstico a plataforma)
├── navalha-cli/     # Interface de linha de comando UNIX com clap derive
└── navalha-web/     # Interface web reativa em Dioxus 0.7.10 (compilada para Wasm)
```

### Princípios Arquiteturais:
1. **Determinismo Bit a Bit**: Os algoritmos não dependem de relógios de sistema, números pseudoaleatórios sem semente fixa ou aproximações probabilísticas. A mesma entrada combinada com os mesmos parâmetros produz exatamente o mesmo array de bytes de saída em qualquer sistema operacional e no navegador.
2. **Zero Dependência Dinâmica de SO**: A rasterização tipográfica, manipulação de imagens e geração de arquivos utilizam crates puras em Rust (`image`, `ab_glyph`, `serde`). A fonte condensada grotesca é embutida no binário via `include_bytes!`, garantindo renderização idêntica onde quer que o binário rode.
3. **Processamento Estritamente Local**: Toda a computação na versão web ocorre na thread do navegador via WebAssembly, sem tráfego de imagens para servidores remotos.

---

## 2. Pipeline de Processamento Gráfico

O fluxo de processamento do sujeito opera em cinco módulos sequenciais e desacoplados:

```
┌────────────────────────────────────────────────────────┐
│                      Foto Bruta                        │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ MÓDULO 1: Guilhotina (Recorte & Canal Alfa)            │
│  - Limiar de luminância ITU-R BT.601                   │
│  - Preservação de máscaras alfa externas               │
│  - Cor sólida de preenchimento opcional                │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ MÓDULO 2: Preparação de Chapa                          │
│  - Downsampling configurável (ex: 600px a 900px)       │
│  - Unsharp Masking agressivo (realce de micro-bordas)  │
│  - Queima de Níveis (Levels: corte de sombra e teto)   │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ MÓDULO 3: Retícula & Dither                            │
│  - Floyd-Steinberg em serpentina alternada             │
│  - Atkinson (dissipação estrita de 75% do erro)        │
│  - Bayer Ordered Dithering (matrizes 4x4 e 8x8)        │
│  - Halftone analógico (0°, 45°, 90°)                   │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ MÓDULO 4: Matriz de Escala                             │
│  - Reescalonamento Nearest-Neighbor estrito            │
│  - Preservação de quinas e bordas táteis de pixel art  │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ MÓDULO 5: Diagramação & Prensa Tipográfica             │
│  - Camada 0: Fundo sólido                              │
│  - Camada 1: Texto de fundo sangrado                   │
│  - Camada 2: Sujeito processado                        │
│  - Camada 3: Textos em tarja com caracteres vazados    │
│  - Camada 4: Riscos toscos, moldura punk e registros   │
└────────────────────────────────────────────────────────┘
```

---

## 3. Detalhamento Algorítmico

### 3.1. Módulo 1 — Guilhotina (`guillotine.rs`)
Calcula a luminância perceptual de cada pixel:
$$L = 0.299 \cdot R + 0.587 \cdot G + 0.114 \cdot B$$
Quando habilitada em modo padrão, isola fundos claros onde $L \ge \text{threshold}$, transformando-os em canais alfa transparentes ($A = 0$). Se a imagem já possuir transparência nativa, ela é preservada integralmente.

### 3.2. Módulo 2 — Preparação de Chapa (`prep.rs`)
1. **Downsampling Forçado**: Reamostra a largura da imagem para um tamanho base reduzido (ex: 720px) usando filtro Lanczos3. A redução de resolução física é crucial para ditar o tamanho tátil do grão da retícula final.
2. **Unsharp Masking Agressivo**:
   - Computa uma versão borrada da imagem via box blur multicanal de duas passadas.
   - Aplica realce severo: $\text{pixel}_{\text{novo}} = \text{orig} + (\text{orig} - \text{blur}) \times \text{strength}$.
   - Isso garante que contornos anatômicos sutis (olhos, vincos e roupas) continuem visíveis após a quantização.
3. **Queima de Níveis (`Levels`)**:
   - Força os valores com luminância abaixo do corte de sombra para zero absoluto.
   - Força os valores com luminância acima do corte de altas luzes para 255.
   - Expande os meios-tons restantes através de mapeamento linear e curva gama.

### 3.3. Módulo 3 — Retícula & Dither (`dither.rs`)

#### A. Floyd-Steinberg com Varredura em Serpentina
Para evitar o aparecimento de padrões direcionais ("minhocas") típicos do dither unidirecional, o algoritmo alterna o sentido de varredura a cada linha:
* **Linhas Pares (Esquerda para Direita):**
  * $(x + 1, y) \leftarrow \frac{7}{16}$
  * $(x - 1, y + 1) \leftarrow \frac{3}{16}$
  * $(x, y + 1) \leftarrow \frac{5}{16}$
  * $(x + 1, y + 1) \leftarrow \frac{1}{16}$
* **Linhas Ímpares (Direita para Esquerda):**
  * $(x - 1, y) \leftarrow \frac{7}{16}$
  * $(x + 1, y + 1) \leftarrow \frac{3}{16}$
  * $(x, y + 1) \leftarrow \frac{5}{16}$
  * $(x - 1, y + 1) \leftarrow \frac{1}{16}$

#### B. Atkinson Dither
Popularizado pela fotocomposição dos computadores Macintosh originais. Diferente do Floyd-Steinberg que distribui 100% do erro residual, o Atkinson distribui apenas $\frac{6}{8} = 75\%$ do erro entre 6 vizinhos (descartando os 25% restantes). Essa perda proposital cria áreas limpas, alto contraste e extrema fidelidade a fanzines xerografados.

#### C. Bayer Ordered Dithering
Utiliza matrizes limiares geométricas rígidas ($M_{4\times 4}$ e $M_{8\times 8}$), gerando uma estética puramente digital dos anos 80 e de monitores CGA/EGA.

#### D. Halftone Analógico
Calcula o ponto de impressão rotacionando as coordenadas cartesianas pelo ângulo de retícula ($\theta \in \{0^\circ, 45^\circ, 90^\circ\}$) e calculando a distância euclidiana ao centro de cada célula da grade de retícula.

### 3.4. Paleta e Preset VGA 256 (Modo 13h)
Projetado para simular fielmente o modo gráfico padrão do MS-DOS dos anos 90:
- **Resolução de Chapa**: Downsampling forçado para uma largura base de 320px (a mesma densidade horizontal do Modo 13h `320x200`).
- **Paleta de 256 Cores**: Composta por 16 cores básicas do padrão BIOS/EGA, cubo RGB 6x6x6 (216 cores) e 24 níveis de cinza monocromáticos.
- **Quantização**: Matriz Bayer 4x4 gerando granulação tátil rígida sem interpolação bilinear.

---

## 4. Diagramação & Prensa Anti-Design (`press.rs`)

O motor de montagem organiza o pôster em 5 camadas físicas:

1. **Camada 0 — Chapa de Fundo**: Cor sólida selecionada da paleta.
2. **Camada 1 — Tipografia de Fundo (Sangria)**: Texto de grande porte ($96\text{pt}+$ com distorção vertical de $180\%\text{ a }220\%$), propositalmente extrapolando a área segura de corte.
3. **Camada 2 — Sujeito Processado**: Imagem recortada e quantizada, centralizada ou alinhada à grade visual. O recorte de silhueta (guilhotina `GuillotineConfig`) suporta edição granular e desacoplada do preset ativo (ativação/desativação manual via `--sem-recorte`/`--recorte`, ajuste contínuo de limiar de luminância `--recorte-threshold` e inversão de máscara `--recorte-inverter`).
4. **Camada 3 — Tarjas Vazadas de Choque**: Blocos nos quais a tipografia é recortada de modo negativo, suportando desativação e remoção total (modo "Sem Faixa", esvaziando `comp.foreground_texts`), quebras de linha (`\n`) com ajuste vertical automático da tarja, seleção de família tipográfica (`BannerFontFamily`: `ImpactBrutal`, `MonoArchive`, `SerifEditorial`), controle granular de tamanho de fonte (`font_size`), auto-fit inteligente (`auto_fit`), reposicionamento livre no canvas (`pos_x`, `pos_y`) e controle de inclinação/ângulo em graus (`rotation`) com rasterização determinística via mapeamento rotacional inverso (preservando nitidez sem furos) e espelhamento vetorial SVG via transformações de rotação (`<g transform="rotate(...)">`), modo de faixa adaptável ao preset ativo com unificação canônica em todos os 20 presets anti-design (`build_pipeline_and_composition` delegando estritamente para `default_banner_element` garantindo paridade total entre CLI, Core e Web), customização plena de cor de frente (`color`) e fundo (`bg_tarja` com suporte a modo transparente `None`), acabamento e estilo de borda (`TarjaBorderFinish`: `Nenhuma`, `Solida`, `Tracejada`, `Picote`, `FitaDurex`), espessura configurável (`border_width`), cor customizada de borda (`border_color`), posicionamento duplo (faixa superior de manchete e faixa inferior de slogan com herança de estilo automática do preset ativo), acabamento geométrico sólido tradicional ou artefatos determinísticos de fotocopiadora (`TarjaEffect::Xerox`: jitter de varredura lateral, riscos de cilindro com perda de toner, micro-cavidades de descolamento de pó e poeira periférica via `lcg_hash`), **distorção não-linear de texto senoidal** (`wave_amplitude` e `wave_wavelength`) para ondulação psicodélica analógica ou sinal CRT (como em `GlitchCrt`, `FanzineDigital` e `LapaNoturna`), e **rotação individual de palavras em recortes de tarja** (`word_tarja_cut` e `word_jitter_rotation`) com quebra particionada de palavras, mini-tarjas independentes e jitter angular orgânico de colagem manual (como em `PunkRiot`, `TrashPolka`, `TabloidePopular` e `AsfaltoAlerta`), enquanto presets estruturais (`XeroxPodre`, `Brutalismo`, `ViagemSolitaria`, etc.) mantêm rigorosamente suas tarjas retas e contínuas sem deformação. Todo o jitter, micro-desalinhamento e fase de onda é **estritamente determinístico**, derivado via `compute_subject_seed` através de hash FNV-1a de 64 bits aplicado sobre o buffer de pixels da imagem de entrada (`processed_subject`).
5. **Camada 4 — Riscos e Marcas de Registro**: Elementos vetoriais agressivos (cruzes de corte em quinas, linhas de corte a laser/lâmina e moldura periférica de 3px).
6. **Camada 5 — Stickers Temáticos e Elementos de Recorte**: Adesivos de protesto e recortes gráficos (`StickerElement`) com suporte tanto a variantes canônicas de vetor/símbolo quanto a upload de imagens personalizadas (`custom_image: Option<Arc<RgbaImage>>`), reamostradas com filtro Nearest Neighbor proporcionalmente ao tamanho configurado, colados sobre a chapa com suporte a rotação angular contínua, filtragem tonal pela paleta do preset e acabamento morfológico de corte/borda (`StickerBorder`: `DieCutBranco` com margem expandida e vinco perimetral de estilete, `Picote` perfurado destacável, borda `Serrilhada` com dentes de serra, `LinhaPreta` de contorno nankin e modo `Nenhuma`), implementado via dilatação morfológica euclidiana determinística (`apply_sticker_border`) com padding dinâmico e renderização em SVG correspondente via serialização pura em Base64 (`bytes_to_base64`).

### 3.6. Módulo 6 — Separação de Chapas para Riso e Silk-Screen (`plates.rs`)

O motor de separação de cores extrai fotolitos analógicos diretamente da composição final:
1. **Histograma e Deduplicação Cromática**:
   - Varre a imagem pixel a pixel agrupando cores com canal alfa opaco ($A > 0$).
   - Computa a cobertura relativa ($\%$) de cada chapa na área total.
2. **Modos de Exibição de Fotolito (`PlateMode`)**:
   - `InsolationMask`: Converte os pixels da cor ativa em preto absoluto (`#000000`) e todo o restante em branco translúcido (`#FFFFFF`), gerando a máscara ideal para insolamento UV em serigrafia ou gravação térmica de master na Risografia.
   - `ColorPreview`: Preserva a cor original sobre fundo transparente para sobreposição de camadas.
   - `ColorOnWhite`: Renderiza a cor original sobre fundo branco para provas de prelo.
3. **Marcas de Registro e Cruzes de Mira (`draw_registration_marks`)**:
   - Desenha cruzes de alinhamento e círculos concêntricos nos 4 cantos e centros das bordas externas com espessura controlada.
4. **Cabeçalho Técnico Informativo (`draw_plate_header`)**:
   - Insere na margem superior a identificação da chapa, nome do arquivo, código hexadecimal, percentual de cobertura e índice sequencial.
5. **Geração Nativa de PDF Multipágina (`generate_plates_pdf`)**:
   - Implementado em Rust puro sem dependências externas de bibliotecas C, compatível tanto com Linux/macOS/Windows nativo quanto com WebAssembly.
   - Emite PDF 1.4 válido com streams comprimidos via `miniz_oxide` (FlateDecode) e tabela de referências cruzadas (`xref`) bit-a-bit determinística.

---


## 5. Simetria CLI e Web & Motor Reativo com Debounce

A interface web escrita em Dioxus 0.7.10 é um espelho dinâmico do compilador CLI com pipeline assíncrono otimizado:
* **Desacoplamento de UI e Computação:** Alterações visuais da interface (troca de abas, visão móvel, status de botões) não disparam reprocessamento de imagem.
* **Debounce Reativo no WebAssembly:** Para campos de entrada contínua (digitação do texto principal ou caixas extras, arraste de sliders de zoom/posição e parâmetros finos), o motor aguarda uma janela de inatividade (250ms via `TimeoutFuture`).
* **Cache em Memória da Camada do Sujeito (`SubjectLayerCache`):** O pipeline desacopla o processamento do sujeito (enquadramento, guilhotina, preparação de chapa, retícula e escala) da composição do pôster. Quando elementos secundários como textos, faixas, stickers ou danos são alterados, o buffer processado do sujeito é resgatado instantaneamente via chave determinística (`SubjectCacheKey`), evitando o reprocessamento pesado da imagem bruta.
* **Controle de Geração e Cancelamento Cooperativo:** Cada interação incrementa uma sequência de versão atômica (`render_seq`). Através de checkpoints cooperativos (`process_subject_cancellable` e `render_poster_cancellable`), qualquer tarefa em andamento aborta sua execução imediatamente no momento exato em que novas entradas do usuário são registradas, eliminando o enfileiramento de processamentos intermediários obsoletos.
* **Feedback Visual Brutalista:** Durante o processamento assíncrono, o pôster anterior permanece estável na tela sem flicker ou tela preta, acompanhado de um indicador temático `[ RENDERIZANDO... ]`.
* O rodapé técnico compõe a string exata do comando `navalha poster` ou `navalha compor` em tempo real.
* O usuário pode visualizar o pôster, clicar no botão de cópia e executar o comando idêntico no terminal ou em scripts CI de produção.

---

## 6. Distribuição na Borda (Cloudflare Workers & Codeberg Pages)

A entrega da aplicação web e da documentação hiperdescritiva opera sem servidores de aplicação dedicados:

```
                  ┌──────────────────────────────────────────────┐
                  │ Push na branch main (Forgejo / Codeberg CI)   │
                  └──────────────────────┬───────────────────────┘
                                         │
                    ┌────────────────────┴────────────────────┐
                    ▼                                         ▼
   ┌─────────────────────────────────┐       ┌─────────────────────────────────┐
   │ Cloudflare Workers (Primary)    │       │ Codeberg Pages (Redundancy)     │
   │  - wrangler.jsonc (site/)       │       │  - Branch 'pages' órfã          │
   │  - navalha.2lp.in               │       │  - Mirror estático instantâneo  │
   │  - / (App) e /doc (Manifesto)   │       └─────────────────────────────────┘
   └─────────────────────────────────┘
```

1. **Topologia de Produção (`site/`)**:
   - `site/index.html`: Shell da aplicação Dioxus com inicializador WebAssembly e metadados Open Graph.
   - `site/doc/index.html`: Documentação exaustiva com histórico e justificativa de cada preset.
   - `site/pkg/`: Binários otimizados de Wasm (`navalha_web_bg.wasm`) e glue code JS.
   - `site/_headers`: Cabeçalhos estritos de segurança e cache HTTP (`X-Content-Type-Options: nosniff`, MIME type de WebAssembly).
2. **Alta Disponibilidade e Redundância**: O workflow CI/CD tenta primeiramente o deploy na rede de borda da Cloudflare (`navalha.2lp.in`). Caso as credenciais não estejam disponíveis ou ocorra indisponibilidade temporária, um fallback automático publica o mesmo diretório na branch `pages` do Codeberg.
