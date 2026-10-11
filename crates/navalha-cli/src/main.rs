#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::type_complexity
)]

use clap::{Parser, Subcommand};
use image::{ImageFormat, ImageReader};
use navalha_core::{
    generate_plates_pdf, generate_poster_svg, process_subject, render_poster, separate_plates,
    AestheticPreset, BannerFontFamily, DamageElement, DitherAlgorithm, GuillotineConfig, Palette,
    PlateMode, PlateSeparationConfig, PosterComposition, PosterFormat, PrepConfig, PrintPlate,
    PunkOverlayConfig, RgbColor, ScaleFactor, StickerBorder, StickerElement, StickerKind,
    TarjaBorderFinish, TarjaEffect, TextElement,
};
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "navalha",
    about = "⚡ Navalha: Motor gráfico anti-design, zine punk e composição de alta fricção visual",
    version,
    author = "Lumen Pink <hi@lumen.pink>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 🚀 Pôster Relâmpago: Entre foto, frase e estética anti-design para obter um pôster pronto instantaneamente
    Poster {
        /// Caminho da foto de entrada (use '-' para ler do stdin)
        input: String,

        /// Frase de impacto do pôster (faixa inferior)
        #[arg(short, long, default_value = "EU SOBREVIVO AO MUNDO")]
        frase: String,

        /// Frase ou manchete da faixa superior (opcional)
        #[arg(long = "frase-superior", aliases = ["texto-topo", "faixa-superior"])]
        frase_superior: Option<String>,

        /// Estética anti-design: punk-riot, navalha-sata, xerox-podre, viagem-solitaria, lapa-noturna, teatro-operario, risografia, fanzine-digital, asfalto-alerta, vga256, cga, old-mac-pop-art, ascii-art, tabloide-popular, brutalismo, trash-polka, atari-gaming, mimeografo, xilogravura, glitch-crt, punk-80-br, anarco-zine, paulete-furacao, symmy-larrat
        #[arg(short, long, default_value = "punk-riot")]
        estetica: String,

        /// Rotação prévia da imagem em graus (0, 90, 180, 270)
        #[arg(long, default_value_t = 0)]
        rotacao: u32,

        /// Fator de zoom granular na foto (ex: 0.5 para afastar, 1.5 para aproximar, padrão 1.0)
        #[arg(long, default_value_t = 1.0)]
        zoom: f32,

        /// Recorte/margem superior da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-top", default_value_t = 0.0)]
        crop_top: f32,

        /// Recorte/margem inferior da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-bottom", default_value_t = 0.0)]
        crop_bottom: f32,

        /// Recorte/margem esquerda da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-left", default_value_t = 0.0)]
        crop_left: f32,

        /// Recorte/margem direita da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-right", default_value_t = 0.0)]
        crop_right: f32,

        /// Deslocamento horizontal do sujeito em pixels (padrão 0 = centralizado)
        #[arg(long = "offset-x", default_value_t = 0)]
        offset_x: i32,

        /// Deslocamento vertical do sujeito em pixels (padrão 0 = centralizado)
        #[arg(long = "offset-y", default_value_t = 0)]
        offset_y: i32,

        /// Danos e marcas de intervenção física. Aceita nome simples (tiro) ou especificador completo
        /// "tipo:posX:posY:escala" onde posição é 0.0–1.0 e escala é multiplicador (1.0 = padrão).
        /// Exemplo: tiro,sangue ou "tiro:0.50:0.40:1.5","sangue:0.30:0.70:2.0"
        #[arg(long, value_delimiter = ',')]
        dano: Vec<String>,

        /// Stickers temáticos. Aceita nome simples (anarchy) ou especificador completo
        /// "tipo:posX:posY:tamanho:rotacao" onde posição é 0.0–1.0, tamanho em px, rotação em graus.
        /// Exemplo: anarchy,skull ou "anarchy:0.75:0.25:140:-15","skull:0.20:0.80:100:5"
        #[arg(long, value_delimiter = ',')]
        sticker: Vec<String>,

        /// Filtrar cores dos stickers com a paleta da estética
        #[arg(long, default_value_t = false)]
        filtrar_stickers: bool,

        /// Estilo de borda de recorte dos stickers: nenhuma, diecut, picote, serrilhada, preta
        #[arg(long = "sticker-borda")]
        sticker_borda: Option<String>,

        /// Espessura da borda de recorte dos stickers em pixels (padrão 4)
        #[arg(long = "sticker-borda-largura", default_value_t = 4)]
        sticker_borda_largura: u32,

        /// Stickers e recortes com imagens personalizadas do usuário.
        /// Formato: "CAMINHO[:posX:posY[:tamanho[:rotacao[:borda[:largura]]]]]"
        /// Exemplo: --sticker-custom "./selo.png:0.5:0.5:120:0:diecut:4"
        #[arg(long = "sticker-custom")]
        sticker_custom: Vec<String>,

        /// Caixas de texto extra: "TEXTO:posX:posY:tamanho:inclinacao:acabamento"
        /// Acabamentos: tarja (padrão), tape, serrilhado, rasgo
        /// Exemplo: --texto-extra "REVOLTA:0.5:0.3:48:10:tape"
        #[arg(long)]
        texto_extra: Vec<String>,

        /// Posição horizontal customizada da faixa principal (0.0 a 1.0)
        #[arg(long = "banner-x")]
        banner_x: Option<f32>,

        /// Posição vertical customizada da faixa principal (0.0 a 1.0)
        #[arg(long = "banner-y")]
        banner_y: Option<f32>,

        /// Inclinação/rotação da faixa principal em graus (-180 a 180)
        #[arg(long = "banner-rot", default_value_t = 0)]
        banner_rot: i16,

        /// Cor do texto da faixa principal (nome de cor ou #hexadecimal)
        #[arg(long = "banner-fg")]
        banner_fg: Option<String>,

        /// Cor de fundo da faixa principal (nome de cor, #hexadecimal ou "none")
        #[arg(long = "banner-bg")]
        banner_bg: Option<String>,

        /// Estilo de borda da faixa principal: nenhuma, solida, tracejada, picote, durex
        #[arg(long = "banner-borda")]
        banner_borda: Option<String>,

        /// Espessura da borda da faixa principal em pixels
        #[arg(long = "banner-borda-largura")]
        banner_borda_largura: Option<u32>,

        /// Cor da borda da faixa principal (nome de cor ou #hexadecimal)
        #[arg(long = "banner-borda-cor")]
        banner_borda_cor: Option<String>,

        /// Efeito/acabamento da tarja: solida, xerox
        #[arg(long = "banner-efeito")]
        banner_efeito: Option<String>,

        /// Família tipográfica da faixa: impact, mono (typewriter/olivetti), serif (jornal), sans
        #[arg(long = "banner-fonte", alias = "fonte")]
        banner_fonte: Option<String>,

        /// Amplitude da distorção ondulada em pixels (0.0 = desativada)
        #[arg(long = "onda-amp")]
        onda_amp: Option<f32>,

        /// Comprimento de onda senoidal em pixels
        #[arg(long = "onda-freq")]
        onda_freq: Option<f32>,

        /// Jitter de rotação individual por palavra em graus (0 = desativado)
        #[arg(long = "palavra-rot")]
        palavra_rot: Option<i16>,

        /// Ativa ou desativa modo de palavras recortadas com mini-tarjas individuais
        #[arg(long = "palavra-tarja")]
        palavra_tarja: Option<bool>,

        /// Modo fanzine / ransom note (colagem de letras/palavras com fontes e cores alternadas)
        #[arg(long = "ransom-note", default_value_t = false)]
        ransom_note: bool,

        /// Fator multiplicador de espaçamento entre palavras em recortes/ransom note (padrão: 1.0)
        #[arg(long = "palavra-espacamento", alias = "word-spacing")]
        palavra_espacamento: Option<f32>,

        /// Fator multiplicador de espaçamento vertical entre linhas (line-height, padrão: 1.0)
        #[arg(long = "linha-espacamento", alias = "line-spacing")]
        linha_espacamento: Option<f32>,

        /// Desativa/remove totalmente as faixas decorativas de texto
        #[arg(long = "sem-faixa", default_value_t = false)]
        sem_faixa: bool,

        /// Desativa o recorte de silhueta (guilhotina), mantendo o fundo original da foto
        #[arg(long = "sem-recorte", default_value_t = false)]
        sem_recorte: bool,

        /// Força ativação ou desativação do recorte de silhueta (guilhotina)
        #[arg(long = "recorte")]
        recorte: Option<bool>,

        /// Limiar manual do recorte de silhueta (0.0 a 1.0)
        #[arg(long = "recorte-threshold")]
        recorte_threshold: Option<f32>,

        /// Inverte a máscara do recorte de silhueta
        #[arg(long = "recorte-inverter", default_value_t = false)]
        recorte_inverter: bool,

        /// Exporta separação de chapas por cor pura para risografia e silk-screen
        #[arg(long = "separar-chapas", default_value_t = false)]
        separar_chapas: bool,

        /// Diretório personalizado para salvar as chapas (opcional)
        #[arg(long = "chapas-dir")]
        chapas_dir: Option<PathBuf>,

        /// Formato e orientação do pôster: vertical (800x1100), horizontal (1100x800), quadrado (1000x1000)
        #[arg(long = "formato", alias = "orientacao", default_value = "vertical")]
        formato: String,

        /// Arquivo de saída (.png ou .svg)
        #[arg(short = 'o', long = "saida", default_value = "poster.png")]
        saida: PathBuf,
    },

    /// ⚙️ Processamento direto da imagem para retícula & dither determinístico puro
    Dither {
        /// Imagem de entrada (ou '-' para stdin)
        input: String,

        /// Largura base de downsampling (dita o tamanho físico do grão)
        #[arg(short, long, default_value_t = 800)]
        resolucao: u32,

        /// Algoritmo de quantização: floyd-steinberg, atkinson, bayer4x4, bayer8x8, halftone
        #[arg(short, long, default_value = "floyd-steinberg")]
        algoritmo: String,

        /// Paleta pré-definida ou lista hexadecimal (ex: classic-punk, vga256, cga ou "#000,#fff,#ff0033")
        #[arg(short, long, default_value = "classic-punk")]
        paleta: String,

        /// Fator de ampliação Nearest-Neighbor estrito (1x, 2x, 3x, 4x)
        #[arg(short, long, default_value = "2x")]
        escala: String,

        /// Caminho do arquivo de saída
        #[arg(short = 'o', long = "saida", default_value = "dither.png")]
        saida: PathBuf,
    },

    /// ✂️ Diagramação e Composição Completa em 5 Camadas
    Compor {
        /// Imagem de entrada (ou '-' para stdin)
        input: String,

        /// Rotação da imagem em graus (0, 90, 180, 270)
        #[arg(long, default_value_t = 0)]
        rotacao: u32,

        /// Fator de zoom granular na foto (ex: 0.5 para afastar, 1.5 para aproximar, padrão 1.0)
        #[arg(long, default_value_t = 1.0)]
        zoom: f32,

        /// Recorte/margem superior da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-top", default_value_t = 0.0)]
        crop_top: f32,

        /// Recorte/margem inferior da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-bottom", default_value_t = 0.0)]
        crop_bottom: f32,

        /// Recorte/margem esquerda da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-left", default_value_t = 0.0)]
        crop_left: f32,

        /// Recorte/margem direita da imagem em porcentagem (0.0 a 45.0)
        #[arg(long = "crop-right", default_value_t = 0.0)]
        crop_right: f32,

        /// Deslocamento horizontal do sujeito em pixels (padrão 0 = centralizado)
        #[arg(long = "offset-x", default_value_t = 0)]
        offset_x: i32,

        /// Deslocamento vertical do sujeito em pixels (padrão 0 = centralizado)
        #[arg(long = "offset-y", default_value_t = 0)]
        offset_y: i32,

        /// Ativa isolamento por guilhotina
        #[arg(long, default_value_t = false)]
        recorte: bool,

        /// Limiar de luminância da guilhotina (0.0 a 1.0)
        #[arg(long, default_value_t = 0.85)]
        recorte_threshold: f32,

        /// Calcular threshold de isolamento automaticamente via algoritmo Otsu
        #[arg(long, default_value_t = false)]
        auto_threshold: bool,

        /// Algoritmo de dither: floyd-steinberg, atkinson, bayer4x4, bayer8x8, halftone
        #[arg(long, default_value = "atkinson")]
        dither: String,

        /// Paleta de cor
        #[arg(long, default_value = "classic-punk")]
        paleta: String,

        /// Fator de escala (1x a 4x)
        #[arg(long, default_value = "2x")]
        escala: String,

        /// Texto maciço de fundo sangrado
        #[arg(long)]
        texto_fundo: Option<String>,

        /// Texto no topo do pôster
        #[arg(long)]
        texto_topo: Option<String>,

        /// Texto principal em tarja sobreposta
        #[arg(long)]
        tarja_baixo: Option<String>,

        /// Distorção vertical da tipografia (ex: 1.5 a 2.2)
        #[arg(long, default_value_t = 1.5)]
        distorcao_vertical: f32,

        /// Modo fanzine / ransom note (colagem de letras/palavras com fontes e cores alternadas)
        #[arg(long = "ransom-note", default_value_t = false)]
        ransom_note: bool,

        /// Fator multiplicador de espaçamento entre palavras em recortes/ransom note (padrão: 1.0)
        #[arg(long = "palavra-espacamento", alias = "word-spacing")]
        palavra_espacamento: Option<f32>,

        /// Fator multiplicador de espaçamento vertical entre linhas (line-height, padrão: 1.0)
        #[arg(long = "linha-espacamento", alias = "line-spacing")]
        linha_espacamento: Option<f32>,

        /// Danos e marcas de intervenção física. Aceita nome simples (tiro) ou especificador completo
        /// "tipo:posX:posY:escala" onde posição é 0.0–1.0 e escala é multiplicador (1.0 = padrão).
        /// Exemplo: tiro,sangue ou "tiro:0.50:0.40:1.5","sangue:0.30:0.70:2.0"
        #[arg(long, value_delimiter = ',')]
        dano: Vec<String>,

        /// Stickers temáticos. Aceita nome simples (anarchy) ou especificador completo
        /// "tipo:posX:posY:tamanho:rotacao" onde posição é 0.0–1.0, tamanho em px, rotação em graus.
        /// Exemplo: anarchy,skull ou "anarchy:0.75:0.25:140:-15","skull:0.20:0.80:100:5"
        #[arg(long, value_delimiter = ',')]
        sticker: Vec<String>,

        /// Estilo de borda de recorte dos stickers: nenhuma, diecut, picote, serrilhada, preta
        #[arg(long = "sticker-borda")]
        sticker_borda: Option<String>,

        /// Espessura da borda de recorte dos stickers em pixels (padrão 4)
        #[arg(long = "sticker-borda-largura", default_value_t = 4)]
        sticker_borda_largura: u32,

        /// Stickers e recortes com imagens personalizadas do usuário.
        /// Formato: "CAMINHO[:posX:posY[:tamanho[:rotacao[:borda[:largura]]]]]"
        /// Exemplo: --sticker-custom "./selo.png:0.5:0.5:120:0:diecut:4"
        #[arg(long = "sticker-custom")]
        sticker_custom: Vec<String>,

        /// Filtrar cores dos stickers com a paleta
        #[arg(long, default_value_t = false)]
        filtrar_stickers: bool,

        /// Caixas de texto extra: "TEXTO:posX:posY:tamanho:inclinacao:acabamento"
        /// Acabamentos: tarja (padrão), tape, serrilhado, rasgo
        /// Exemplo: --texto-extra "REVOLTA:0.5:0.3:48:10:tape"
        #[arg(long)]
        texto_extra: Vec<String>,

        /// Exporta separação de chapas por cor pura para risografia e silk-screen
        #[arg(long = "separar-chapas", default_value_t = false)]
        separar_chapas: bool,

        /// Diretório personalizado para salvar as chapas (opcional)
        #[arg(long = "chapas-dir")]
        chapas_dir: Option<PathBuf>,

        /// Formato e orientação do pôster: vertical (800x1100), horizontal (1100x800), quadrado (1000x1000)
        #[arg(long = "formato", alias = "orientacao", default_value = "vertical")]
        formato: String,

        /// Arquivo de saída (.png ou .svg)
        #[arg(short = 'o', long = "saida", default_value = "poster_composto.png")]
        saida: PathBuf,
    },

    /// 🖨️ Separação de Chapas para Impressão (Risografia e Silk-Screen)
    Chapas {
        /// Imagem ou pôster de entrada (ou '-' para stdin)
        input: String,

        /// Formato de exportação: png, pdf ou ambos (padrão: ambos)
        #[arg(short, long, default_value = "ambos")]
        formato: String,

        /// Modo visual da chapa: fotolito (P&B para matriz), colorido ou branco
        #[arg(short, long, default_value = "fotolito")]
        modo: String,

        /// Desativa as marcas de registro/cruzes de mira nas margens
        #[arg(long, default_value_t = false)]
        sem_registro: bool,

        /// Desativa o cabeçalho técnico informativo
        #[arg(long, default_value_t = false)]
        sem_cabecalho: bool,

        /// Margem extra em pixels para as marcas de registro e texto (padrão 50)
        #[arg(long, default_value_t = 50)]
        margem: u32,

        /// Diretório de saída para salvar os arquivos
        #[arg(short = 'o', long = "saida", default_value = "chapas")]
        saida: PathBuf,
    },

    /// 📟 Conversão em ASCII Art pura no terminal (Waldemar Cordeiro 1971)
    Ascii {
        /// Imagem de entrada (ou '-' para stdin)
        input: String,

        /// Número de colunas no terminal (padrão: 80)
        #[arg(short, long, default_value_t = 80)]
        colunas: u32,

        /// Inverter densidade ótica (ideal para fundos claros/impressão em papel)
        #[arg(short, long)]
        inverter: bool,
    },

    /// 📦 Processamento em lote de diretórios inteiros de imagens
    #[command(alias = "batch")]
    Lote {
        /// Diretório ou arquivo de entrada contendo as imagens
        input: PathBuf,

        /// Diretório de destino para os arquivos gerados
        #[arg(short = 'o', long = "saida", default_value = "lote_posters")]
        saida: PathBuf,

        /// Modo de processamento: poster (pôster anti-design completo) ou dither (retícula pura)
        #[arg(short = 'm', long = "modo", default_value = "poster")]
        modo: String,

        /// Estética anti-design para modo pôster (punk-riot, navalha-sata, xerox-podre, etc.)
        #[arg(short = 'e', long = "estetica", default_value = "punk-riot")]
        estetica: String,

        /// Frase padrão da faixa inferior para todos os pôsteres
        #[arg(short = 'f', long = "frase", default_value = "EU SOBREVIVO AO MUNDO")]
        frase: String,

        /// Extrai a frase do pôster a partir do nome do arquivo (ex: 'contra-cultura.jpg' -> 'CONTRA CULTURA')
        #[arg(long = "frase-do-nome", default_value_t = false)]
        frase_do_nome: bool,

        /// Arquivo .txt contendo frases (uma por linha) para distribuir entre as imagens
        #[arg(long = "frases-arquivo")]
        frases_arquivo: Option<PathBuf>,

        /// Frase ou manchete opcional da faixa superior
        #[arg(long = "frase-superior")]
        frase_superior: Option<String>,

        /// Desativa/remove as faixas de texto no modo pôster
        #[arg(long = "sem-faixa", default_value_t = false)]
        sem_faixa: bool,

        /// Ativa modo fanzine / ransom note (colagem de letras/palavras)
        #[arg(long = "ransom-note", default_value_t = false)]
        ransom_note: bool,

        /// Varrer subdiretórios recursivamente mantendo estrutura
        #[arg(short = 'r', long = "recursivo", default_value_t = false)]
        recursivo: bool,

        /// Lista de extensões aceitas separadas por vírgula (padrão: jpg,jpeg,png,webp,bmp)
        #[arg(long = "extensoes", default_value = "jpg,jpeg,png,webp,bmp")]
        extensoes: String,

        /// Algoritmo de quantização (quando no modo dither): floyd-steinberg, atkinson, bayer4x4, bayer8x8, halftone
        #[arg(long = "algoritmo", default_value = "atkinson")]
        algoritmo: String,

        /// Paleta de cor (quando no modo dither ou override)
        #[arg(long = "paleta", default_value = "classic-punk")]
        paleta: String,

        /// Fator de ampliação (quando no modo dither): 1x, 2x, 3x, 4x
        #[arg(long = "escala", default_value = "2x")]
        escala: String,

        /// Separa chapas para risografia e silk-screen para cada imagem
        #[arg(long = "separar-chapas", default_value_t = false)]
        separar_chapas: bool,

        /// Número de threads simultâneas para processamento paralelo (0 = automático por núcleos de CPU)
        #[arg(short = 'j', long = "jobs", default_value_t = 0)]
        jobs: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Poster {
            input,
            frase,
            frase_superior,
            estetica,
            rotacao,
            zoom,
            crop_top,
            crop_bottom,
            crop_left,
            crop_right,
            offset_x,
            offset_y,
            dano,
            sticker,
            sticker_custom,
            filtrar_stickers,
            sticker_borda,
            sticker_borda_largura,
            texto_extra,
            banner_x,
            banner_y,
            banner_rot,
            banner_fg,
            banner_bg,
            banner_borda,
            banner_borda_largura,
            banner_borda_cor,
            banner_efeito,
            banner_fonte,
            onda_amp,
            onda_freq,
            palavra_rot,
            palavra_tarja,
            ransom_note,
            palavra_espacamento,
            linha_espacamento,
            sem_faixa,
            sem_recorte,
            recorte,
            recorte_threshold,
            recorte_inverter,
            separar_chapas,
            chapas_dir,
            formato,
            saida,
        } => {
            eprintln!("[navalha] ⚡ Afiando a lâmina anti-design...");
            let preset = AestheticPreset::parse(&estetica)
                .unwrap_or_else(|| {
                    eprintln!("[navalha] ⚠️ Estética '{}' desconhecida. Usando PunkRiot.", estetica);
                    AestheticPreset::PunkRiot
                });

            let img = load_image(&input)?;
            if let Some(ref topo) = frase_superior {
                eprintln!(
                    "[navalha] 🔪 Aplicando preset '{}' com manchete: \"{}\" e frase: \"{}\"",
                    preset.display_name(),
                    topo,
                    frase
                );
            } else {
                eprintln!("[navalha] 🔪 Aplicando preset '{}' com frase: \"{}\"", preset.display_name(), frase);
            }

            let (mut g, mut pr, dt, pal, sc, mut comp) =
                preset.build_pipeline_and_composition_dual(&frase, frase_superior.as_deref());
            let poster_fmt = PosterFormat::parse(&formato).unwrap_or(PosterFormat::Vertical);
            comp.set_format(poster_fmt);
            pr.rotation = rotacao;
            pr.zoom = zoom;
            pr.crop_top = crop_top;
            pr.crop_bottom = crop_bottom;
            pr.crop_left = crop_left;
            pr.crop_right = crop_right;
            comp.subject_offset_x = offset_x;
            comp.subject_offset_y = offset_y;
            comp.custom_seed = Some(navalha_core::compute_subject_seed(Some(&img), &frase));

            if ransom_note {
                for t in comp.foreground_texts.iter_mut() {
                    t.ransom_note = true;
                }
            }

            if sem_faixa {
                comp.foreground_texts.clear();
            }

            if sem_recorte {
                g.enabled = false;
            } else if let Some(rec) = recorte {
                g.enabled = rec;
            }
            if let Some(th) = recorte_threshold {
                g.threshold = th;
            }
            if recorte_inverter {
                g.invert = !g.invert;
            }

            let b_idx = if frase_superior.is_some() { 1 } else { 0 };
            if let Some(t) = comp.foreground_texts.get_mut(b_idx) {
                if let Some(bx) = banner_x {
                    t.pos_x = bx;
                }
                if let Some(by) = banner_y {
                    t.pos_y = by;
                }
                if banner_rot != 0 {
                    t.rotation = banner_rot;
                }
                if let Some(ref fg) = banner_fg {
                    if let Some(col) = RgbColor::parse_color_str(fg) {
                        t.color = col;
                    }
                }
                if let Some(ref bg) = banner_bg {
                    if bg.trim().eq_ignore_ascii_case("none") || bg.trim().eq_ignore_ascii_case("sem") {
                        t.bg_tarja = None;
                    } else if let Some(col) = RgbColor::parse_color_str(bg) {
                        t.bg_tarja = Some(col);
                    }
                }
                if let Some(ref b) = banner_borda {
                    if let Some(finish) = TarjaBorderFinish::parse(b) {
                        t.border_finish = finish;
                    }
                }
                if let Some(bw) = banner_borda_largura {
                    t.border_width = bw;
                }
                if let Some(ref bc) = banner_borda_cor {
                    if let Some(col) = RgbColor::parse_color_str(bc) {
                        t.border_color = Some(col);
                    }
                }
                if let Some(ref eff) = banner_efeito {
                    if let Some(ef) = TarjaEffect::parse(eff) {
                        t.tarja_effect = ef;
                    }
                }
                if let Some(amp) = onda_amp {
                    t.wave_amplitude = amp;
                }
                if let Some(freq) = onda_freq {
                    t.wave_wavelength = freq;
                }
                if let Some(rot_p) = palavra_rot {
                    t.word_jitter_rotation = rot_p;
                    if rot_p > 0 {
                        t.word_tarja_cut = true;
                    }
                }
                if let Some(cut) = palavra_tarja {
                    t.word_tarja_cut = cut;
                }
                if let Some(sp) = palavra_espacamento {
                    t.word_spacing = sp;
                }
                if let Some(lsp) = linha_espacamento {
                    t.line_spacing = lsp;
                }
            }

            if let Some(ref bf) = banner_fonte {
                if let Some(fam) = BannerFontFamily::parse(bf) {
                    for t in comp.foreground_texts.iter_mut() {
                        t.font_family = fam;
                    }
                }
            }

            // Parsing de danos: suporta "tiro" (padrão) ou "tiro:posX:posY:escala" (customizado)
            for d_spec in &dano {
                if let Some(elem) = DamageElement::parse_spec(d_spec) {
                    let ja_existe = comp.overlay.damage_elements.iter().any(|e| e.kind == elem.kind);
                    if !ja_existe {
                        comp.overlay.damage_elements.push(elem);
                    }
                }
            }

            // Parsing de stickers: suporta "anarchy" (padrão) ou "anarchy:posX:posY:tamanho:rotacao[:borda[:largura]]" (customizado)
            let def_stk_border = sticker_borda
                .as_deref()
                .and_then(StickerBorder::parse)
                .unwrap_or(StickerBorder::DieCutBranco);

            for stk_spec in &sticker {
                if let Some(stk) = parse_sticker_spec(stk_spec, filtrar_stickers, def_stk_border, sticker_borda_largura) {
                    comp.stickers.push(stk);
                }
            }

            for custom_spec in &sticker_custom {
                if let Some(stk) = parse_custom_sticker_spec(custom_spec, filtrar_stickers, def_stk_border, sticker_borda_largura) {
                    comp.stickers.push(stk);
                }
            }

            for spec in &texto_extra {
                if let Some(el) = parse_texto_extra(spec) {
                    comp.foreground_texts.push(el);
                }
            }

            let processed = process_subject(&img, &g, &pr, dt, &pal, sc);
            let poster = render_poster(&comp, Some(&processed));

            save_output(&poster, &comp, &saida)?;
            eprintln!("[navalha] 💀 Pôster gerado com sucesso em: {}", saida.display());

            if separar_chapas {
                let out_dir = chapas_dir.unwrap_or_else(|| {
                    let stem = saida.file_stem().unwrap_or_default().to_string_lossy();
                    saida.parent().unwrap_or_else(|| Path::new(".")).join(format!("{}_chapas", stem))
                });
                let plate_cfg = PlateSeparationConfig::default();
                let plates = separate_plates(&poster, &plate_cfg)?;
                eprintln!("[navalha] 🖨️ Separando {} chapas de impressão (Risografia / Silk-screen)...", plates.len());
                export_plates_to_dir(&plates, &out_dir, "ambos")?;
            }
        }

        Commands::Dither {
            input,
            resolucao,
            algoritmo,
            paleta,
            escala,
            saida,
        } => {
            eprintln!("[navalha] ⚙️ Processando retícula direta...");
            let img = load_image(&input)?;
            let palette = resolve_palette(&paleta);
            let dither_algo = DitherAlgorithm::parse(&algoritmo).unwrap_or(DitherAlgorithm::FloydSteinberg);
            let scale_factor = ScaleFactor::parse(&escala).unwrap_or(ScaleFactor::TwoX);

            let prep = PrepConfig {
                target_width: Some(resolucao),
                ..Default::default()
            };
            let guillotine = GuillotineConfig::default();

            let out = process_subject(&img, &guillotine, &prep, dither_algo, &palette, scale_factor);
            out.save_with_format(&saida, ImageFormat::Png)?;
            eprintln!("[navalha] 🖨️ Retícula finalizada: {}", saida.display());
        }

        Commands::Compor {
            input,
            rotacao,
            zoom,
            crop_top,
            crop_bottom,
            crop_left,
            crop_right,
            offset_x,
            offset_y,
            recorte,
            recorte_threshold,
            auto_threshold,
            dither,
            paleta,
            escala,
            texto_fundo,
            texto_topo,
            tarja_baixo,
            distorcao_vertical,
            ransom_note,
            palavra_espacamento,
            linha_espacamento,
            dano,
            sticker,
            sticker_borda,
            sticker_borda_largura,
            sticker_custom,
            filtrar_stickers,
            texto_extra,
            separar_chapas,
            chapas_dir,
            formato,
            saida,
        } => {
            eprintln!("[navalha] 📐 Montando prensagem brutalista em 5 camadas...");
            let img = load_image(&input)?;
            let palette = resolve_palette(&paleta);
            let dither_algo = DitherAlgorithm::parse(&dither).unwrap_or(DitherAlgorithm::Atkinson);
            let scale_factor = ScaleFactor::parse(&escala).unwrap_or(ScaleFactor::TwoX);

            let guillotine = GuillotineConfig {
                enabled: recorte,
                threshold: recorte_threshold,
                auto_threshold,
                invert: true,
                background_fill: None,
            };
            let prep = PrepConfig {
                rotation: rotacao,
                zoom,
                crop_top,
                crop_bottom,
                crop_left,
                crop_right,
                ..Default::default()
            };

            let processed = process_subject(&img, &guillotine, &prep, dither_algo, &palette, scale_factor);

            let mut fg_texts = Vec::new();
            if let Some(topo) = texto_topo {
                fg_texts.push(TextElement {
                    text: topo.to_uppercase(),
                    font_size: 44.0,
                    stretch_x: 1.0,
                    stretch_y: distorcao_vertical,
                    color: RgbColor::new(255, 255, 255),
                    bg_tarja: Some(RgbColor::new(0, 0, 0)),
                    tarja_padding: 10,
                    pos_x: 0.5,
                    pos_y: 0.08,
                    center_x: true,
                    rotation: 0,
                    tarja_effect: TarjaEffect::Solida,
                    ..Default::default()
                });
            }

            if let Some(ref baixo) = tarja_baixo {
                fg_texts.push(TextElement {
                    text: baixo.to_uppercase(),
                    font_size: 48.0,
                    stretch_x: 1.0,
                    stretch_y: distorcao_vertical,
                    color: RgbColor::new(255, 255, 255),
                    bg_tarja: Some(RgbColor::new(0, 0, 0)),
                    tarja_padding: 12,
                    pos_x: 0.5,
                    pos_y: 0.88,
                    center_x: true,
                    rotation: 0,
                    tarja_effect: TarjaEffect::Solida,
                    ransom_note,
                    word_spacing: palavra_espacamento.unwrap_or(1.0),
                    line_spacing: linha_espacamento.unwrap_or(1.0),
                    ..Default::default()
                });
            }

            let bg_element = texto_fundo.map(|txt| TextElement {
                text: txt.to_uppercase(),
                font_size: 88.0,
                stretch_x: 1.2,
                stretch_y: distorcao_vertical * 1.3,
                color: RgbColor::new(220, 220, 220),
                bg_tarja: None,
                tarja_padding: 0,
                pos_x: 0.5,
                pos_y: 0.18,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                ..Default::default()
            });

            let mut overlay = PunkOverlayConfig::default();
            // Parsing de danos: suporta "tiro" (padrão) ou "tiro:posX:posY:escala" (customizado)
            for d_spec in &dano {
                if let Some(elem) = DamageElement::parse_spec(d_spec) {
                    let ja_existe = overlay.damage_elements.iter().any(|e| e.kind == elem.kind);
                    if !ja_existe {
                        overlay.damage_elements.push(elem);
                    }
                }
            }

            // Parsing de stickers: suporta "anarchy" (padrão) ou "anarchy:posX:posY:tamanho:rotacao[:borda[:largura]]" (customizado)
            let def_stk_border = sticker_borda
                .as_deref()
                .and_then(StickerBorder::parse)
                .unwrap_or(StickerBorder::DieCutBranco);

            let mut stickers = Vec::new();
            for stk_spec in &sticker {
                if let Some(stk) = parse_sticker_spec(stk_spec, filtrar_stickers, def_stk_border, sticker_borda_largura) {
                    stickers.push(stk);
                }
            }

            for custom_spec in &sticker_custom {
                if let Some(stk) = parse_custom_sticker_spec(custom_spec, filtrar_stickers, def_stk_border, sticker_borda_largura) {
                    stickers.push(stk);
                }
            }

            for spec in &texto_extra {
                if let Some(el) = parse_texto_extra(spec) {
                    fg_texts.push(el);
                }
            }

            let poster_fmt = PosterFormat::parse(&formato).unwrap_or(PosterFormat::Vertical);
            let comp = PosterComposition {
                canvas_width: 800,
                canvas_height: 1100,
                background_color: RgbColor::new(245, 242, 235),
                background_text: bg_element,
                foreground_texts: fg_texts,
                overlay,
                stickers,
                manual_drawings: Vec::new(),
                palette: Some(palette.clone()),
                subject_offset_x: offset_x,
                subject_offset_y: offset_y,
                custom_seed: Some(navalha_core::compute_subject_seed(Some(&img), tarja_baixo.as_deref().unwrap_or(""))),
            }.with_format(poster_fmt);

            let poster = navalha_core::render_poster(&comp, Some(&processed));
            save_output(&poster, &comp, &saida)?;
            eprintln!("[navalha] ⚡ Composição gravada em: {}", saida.display());

            if separar_chapas {
                let out_dir = chapas_dir.unwrap_or_else(|| {
                    let stem = saida.file_stem().unwrap_or_default().to_string_lossy();
                    saida.parent().unwrap_or_else(|| Path::new(".")).join(format!("{}_chapas", stem))
                });
                let plate_cfg = PlateSeparationConfig::default();
                let plates = separate_plates(&poster, &plate_cfg)?;
                eprintln!("[navalha] 🖨️ Separando {} chapas de impressão (Risografia / Silk-screen)...", plates.len());
                export_plates_to_dir(&plates, &out_dir, "ambos")?;
            }
        }

        Commands::Chapas {
            input,
            formato,
            modo,
            sem_registro,
            sem_cabecalho,
            margem,
            saida,
        } => {
            eprintln!("[navalha] 🖨️ Analisando cores e gerando matrizes de impressão...");
            let img = load_image(&input)?;
            let plate_mode = PlateMode::parse(&modo).unwrap_or(PlateMode::InsolationMask);
            let plate_cfg = PlateSeparationConfig {
                mode: plate_mode,
                registration_marks: !sem_registro,
                margin_px: margem,
                include_header: !sem_cabecalho,
                min_coverage_threshold: 0.0005,
            };

            let plates = separate_plates(&img, &plate_cfg)?;
            eprintln!("[navalha] 📄 Identificadas {} chapas de cor pura na arte:", plates.len());
            for p in &plates {
                eprintln!("   - Chapa #{}: {} ({:.1}% de cobertura)", p.index, p.color_hex, p.coverage_percentage);
            }

            export_plates_to_dir(&plates, &saida, &formato.to_lowercase())?;
            eprintln!("[navalha] ⚡ Chapas exportadas com sucesso no diretório: {}", saida.display());
        }

        Commands::Ascii {
            input,
            colunas,
            inverter,
        } => {
            let rgba = load_image(&input)?;
            let dyn_img = image::DynamicImage::ImageRgba8(rgba);
            let ascii_str = navalha_core::image_to_ascii(&dyn_img, colunas, inverter);
            print!("{}", ascii_str);
        }

        Commands::Lote {
            input,
            saida,
            modo,
            estetica,
            frase,
            frase_do_nome,
            frases_arquivo,
            frase_superior,
            sem_faixa,
            ransom_note,
            recursivo,
            extensoes,
            algoritmo,
            paleta,
            escala,
            separar_chapas,
            jobs,
        } => {
            run_lote(
                input,
                saida,
                modo,
                estetica,
                frase,
                frase_do_nome,
                frases_arquivo,
                frase_superior,
                sem_faixa,
                ransom_note,
                recursivo,
                extensoes,
                algoritmo,
                paleta,
                escala,
                separar_chapas,
                jobs,
            )?;
        }
    }

    Ok(())
}

/// Extrai uma frase limpa em caixa alta a partir do nome do arquivo (ex: 'punk-riot_01' -> 'PUNK RIOT 01').
fn extract_phrase_from_stem(stem: &str) -> String {
    let clean = stem
        .chars()
        .map(|c| if c == '-' || c == '_' || c == '.' { ' ' } else { c })
        .collect::<String>();
    let result = clean
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase();
    if result.is_empty() {
        String::from("EU SOBREVIVO AO MUNDO")
    } else {
        result
    }
}

/// Varre arquivos de imagem com extensões especificadas, de forma determinística e opcionalmente recursiva.
fn collect_image_files(
    root: &Path,
    current: &Path,
    recursive: bool,
    extensions: &[String],
    results: &mut Vec<(PathBuf, PathBuf)>,
) -> std::io::Result<()> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_image_files(root, &path, recursive, extensions, results)?;
            }
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
                    results.push((path, rel));
                }
            }
        }
    }
    Ok(())
}

/// Processa um único item do lote (pôster ou retícula).
fn process_single_item(
    in_path: &Path,
    out_path: &Path,
    modo: &str,
    estetica: &str,
    frase: &str,
    frase_superior: Option<&str>,
    sem_faixa: bool,
    ransom_note: bool,
    algoritmo: &str,
    paleta: &str,
    escala: &str,
    separar_chapas: bool,
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let img_bytes = std::fs::read(in_path)?;
    let reader = ImageReader::new(Cursor::new(img_bytes)).with_guessed_format()?;
    let img = reader.decode()?.to_rgba8();

    if modo.eq_ignore_ascii_case("dither") {
        let palette = resolve_palette(paleta);
        let dither_algo = DitherAlgorithm::parse(algoritmo).unwrap_or(DitherAlgorithm::Atkinson);
        let scale_factor = ScaleFactor::parse(escala).unwrap_or(ScaleFactor::TwoX);
        let prep = PrepConfig::default();
        let guillotine = GuillotineConfig::default();
        let out = process_subject(&img, &guillotine, &prep, dither_algo, &palette, scale_factor);
        out.save_with_format(out_path, ImageFormat::Png)?;
    } else {
        let preset = AestheticPreset::parse(estetica).unwrap_or(AestheticPreset::PunkRiot);
        let (_g, _pr, _dt, _pal, _sc, mut comp) =
            preset.build_pipeline_and_composition_dual(frase, frase_superior);
        comp.custom_seed = Some(navalha_core::compute_subject_seed(Some(&img), frase));

        if sem_faixa {
            comp.foreground_texts.clear();
        }

        if ransom_note {
            for t in comp.foreground_texts.iter_mut() {
                t.ransom_note = true;
            }
        }

        let poster = render_poster(&comp, Some(&img));
        poster.save_with_format(out_path, ImageFormat::Png)?;

        if separar_chapas {
            let stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("item");
            let chapas_dir = target_dir.join(format!("{}_chapas", stem));
            let plate_cfg = PlateSeparationConfig::default();
            let plates = separate_plates(&poster, &plate_cfg)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;
            export_plates_to_dir(&plates, &chapas_dir, "ambos")
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;
        }
    }

    Ok(())
}

/// Orquestra a execução concorrente em lote de todas as imagens encontradas.
fn run_lote(
    input: PathBuf,
    saida: PathBuf,
    modo: String,
    estetica: String,
    frase: String,
    frase_do_nome: bool,
    frases_arquivo: Option<PathBuf>,
    frase_superior: Option<String>,
    sem_faixa: bool,
    ransom_note: bool,
    recursivo: bool,
    extensoes: String,
    algoritmo: String,
    paleta: String,
    escala: String,
    separar_chapas: bool,
    jobs: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[navalha] 📦 Iniciando processamento em lote...");
    let start_all = Instant::now();

    // 1. Lê frases de arquivo se fornecido
    let frases_list: Vec<String> = if let Some(ref f_path) = frases_arquivo {
        let content = std::fs::read_to_string(f_path)?;
        content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // 2. Parser das extensões aceitas
    let exts: Vec<String> = extensoes
        .split(',')
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // 3. Coleta os arquivos de imagem
    let mut files = Vec::new();
    if input.is_file() {
        let filename = PathBuf::from(input.file_name().unwrap_or_default());
        files.push((input.clone(), filename));
    } else if input.is_dir() {
        collect_image_files(&input, &input, recursivo, &exts, &mut files)?;
    } else {
        return Err(format!("Caminho de entrada inválido ou inexistente: {}", input.display()).into());
    }

    // Ordenação alfabética para determinismo estrito
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let total = files.len();
    if total == 0 {
        eprintln!(
            "[navalha] ⚠️ Nenhuma imagem encontrada em '{}' com extensões [{}]",
            input.display(),
            exts.join(", ")
        );
        return Ok(());
    }

    std::fs::create_dir_all(&saida)?;
    eprintln!(
        "[navalha] ⚡ {} imagens encontradas para lote (modo: {}, destino: '{}')",
        total,
        modo,
        saida.display()
    );

    // 4. Concorrência paralela com threads nativas
    let num_threads = if jobs > 0 {
        jobs
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
    .max(1)
    .min(total);

    let next_file_idx = AtomicUsize::new(0);
    let success_counter = AtomicUsize::new(0);
    let error_counter = AtomicUsize::new(0);
    let print_mutex = Mutex::new(());

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                loop {
                    let idx = next_file_idx.fetch_add(1, Ordering::SeqCst);
                    if idx >= total {
                        break;
                    }
                    let (ref in_path, ref rel_path) = files[idx];
                    let start_item = Instant::now();
                    let stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("imagem");

                    let mut target_dir = saida.clone();
                    if let Some(parent) = rel_path.parent() {
                        if !parent.as_os_str().is_empty() {
                            target_dir = target_dir.join(parent);
                        }
                    }
                    let _ = std::fs::create_dir_all(&target_dir);

                    let target_name = format!("{}.png", stem);
                    let mut out_path = target_dir.join(&target_name);
                    if out_path == *in_path {
                        out_path = target_dir.join(format!("{}_poster.png", stem));
                    }

                    let item_phrase = if frase_do_nome {
                        extract_phrase_from_stem(stem)
                    } else if !frases_list.is_empty() {
                        frases_list[idx % frases_list.len()].clone()
                    } else {
                        frase.clone()
                    };

                    let res = process_single_item(
                        in_path,
                        &out_path,
                        &modo,
                        &estetica,
                        &item_phrase,
                        frase_superior.as_deref(),
                        sem_faixa,
                        ransom_note,
                        &algoritmo,
                        &paleta,
                        &escala,
                        separar_chapas,
                        &target_dir,
                    );

                    let elapsed = start_item.elapsed();
                    let _lock = print_mutex.lock().unwrap();
                    match res {
                        Ok(()) => {
                            success_counter.fetch_add(1, Ordering::SeqCst);
                            eprintln!(
                                "[{}/{}] {} -> {} [{:.0?}]",
                                idx + 1,
                                total,
                                in_path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                                out_path.display(),
                                elapsed
                            );
                        }
                        Err(e) => {
                            error_counter.fetch_add(1, Ordering::SeqCst);
                            eprintln!(
                                "[{}/{}] ⚠️ Falha ao processar {}: {}",
                                idx + 1,
                                total,
                                in_path.display(),
                                e
                            );
                        }
                    }
                }
            });
        }
    });

    let total_elapsed = start_all.elapsed();
    let succ = success_counter.load(Ordering::SeqCst);
    let err = error_counter.load(Ordering::SeqCst);
    let rate = if total_elapsed.as_secs_f32() > 0.0 {
        (succ as f32) / total_elapsed.as_secs_f32()
    } else {
        succ as f32
    };

    eprintln!(
        "[navalha] ✅ Lote finalizado: {}/{} imagens processadas com sucesso ({} falhas) em {:.2}s ({:.1} img/s)",
        succ,
        total,
        err,
        total_elapsed.as_secs_f32(),
        rate
    );

    Ok(())
}

/// Converte spec "TEXTO:posX:posY:tamanho:inclinacao:acabamento" em TextElement.
/// Acabamentos suportados: tarja (padrão), tape, serrilhado, rasgo.
fn parse_texto_extra(spec: &str) -> Option<TextElement> {
    let partes: Vec<&str> = spec.splitn(6, ':').collect();
    let texto = partes.first().copied().unwrap_or("").to_uppercase();
    if texto.is_empty() { return None; }

    let pos_x  = partes.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.5);
    let pos_y  = partes.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.5);
    let tamanho = partes.get(3).and_then(|s| s.parse::<f32>().ok()).unwrap_or(48.0);
    let inclinacao = partes.get(4).and_then(|s| s.parse::<i16>().ok()).unwrap_or(0);
    let acabamento = partes.get(5).copied().unwrap_or("tarja");

    // Define aparência conforme acabamento
    let (cor_texto, bg_tarja, padding, tarja_effect) = match acabamento.trim().to_lowercase().as_str() {
        // Xerox: tarja de toner oxidado com riscos e falhas de fotocopiadora
        "xerox" | "xerox-podre" => (
            RgbColor::new(245, 242, 235),
            Some(RgbColor::new(28, 25, 23)),
            14u32,
            TarjaEffect::Xerox,
        ),
        // Tape: tarja cinza claro, texto escuro — evoca fita adesiva
        "tape" => (
            RgbColor::new(20, 20, 20),
            Some(RgbColor::new(200, 200, 180)),
            6u32,
            TarjaEffect::Solida,
        ),
        // Serrilhado: tarja amarelo-neon brutalista, texto preto — borda dentada visual
        "serrilhado" => (
            RgbColor::new(0, 0, 0),
            Some(RgbColor::new(255, 230, 0)),
            10u32,
            TarjaEffect::Solida,
        ),
        // Rasgo: sem tarja, texto vermelho agressivo — simula letra rasgada
        "rasgo" => (
            RgbColor::new(200, 0, 0),
            None,
            0u32,
            TarjaEffect::Solida,
        ),
        // Tarja (padrão): clássico punk — tarja preta, texto branco
        _ => (
            RgbColor::new(255, 255, 255),
            Some(RgbColor::new(0, 0, 0)),
            10u32,
            TarjaEffect::Solida,
        ),
    };

    Some(TextElement {
        text: texto,
        font_size: tamanho,
        stretch_x: 1.0,
        stretch_y: 1.4,
        color: cor_texto,
        bg_tarja,
        tarja_padding: padding,
        pos_x,
        pos_y,
        center_x: true,
        rotation: inclinacao,
        tarja_effect,
        ..Default::default()
    })
}

/// Posições automáticas para stickers quando o usuário não especifica coordenadas.
static STICKER_AUTO_POSITIONS: &[(f32, f32, i16)] = &[
    (0.84, 0.16, -10),
    (0.16, 0.82, 12),
    (0.16, 0.18, -6),
    (0.84, 0.78, 8),
    (0.85, 0.48, -12),
    (0.15, 0.48, 14),
];

/// Contador global simples (thread_local) para índice de posição automática.
use std::cell::Cell;
thread_local! {
    static STICKER_AUTO_IDX: Cell<usize> = const { Cell::new(0) };
}

/// Converte spec "tipo:posX:posY:tamanho:rotacao[:borda[:largura]]" (ou apenas "tipo") em StickerElement.
///
/// Quando omitidos, posição e tamanho são atribuídos automaticamente de forma determinística.
fn parse_sticker_spec(
    spec: &str,
    apply_palette: bool,
    default_border: StickerBorder,
    default_border_width: u32,
) -> Option<StickerElement> {
    let partes: Vec<&str> = spec.splitn(7, ':').collect();
    let kind = StickerKind::parse(partes.first().copied().unwrap_or(""))?;

    let element = if partes.len() >= 3 {
        // Sintaxe completa: tipo:posX:posY[:tamanho[:rotacao[:borda[:largura]]]]
        let pos_x = partes[1].parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0);
        let pos_y = partes[2].parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0);
        let size = partes.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(100);
        let rotation = partes.get(4).and_then(|s| s.parse::<i16>().ok()).unwrap_or(0);
        let border = partes.get(5).and_then(|s| StickerBorder::parse(s)).unwrap_or(default_border);
        let border_width = partes.get(6).and_then(|s| s.parse::<u32>().ok()).unwrap_or(default_border_width);
        StickerElement::new(kind, pos_x, pos_y, size, rotation, apply_palette)
            .with_border(border, border_width)
    } else {
        // Sintaxe simples: usa posição automática determinística
        let idx = STICKER_AUTO_IDX.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v
        });
        let (px, py, rot) = STICKER_AUTO_POSITIONS[idx % STICKER_AUTO_POSITIONS.len()];
        StickerElement::new(kind, px, py, 100, rot, apply_palette)
            .with_border(default_border, default_border_width)
    };
    Some(element)
}

/// Converte spec de sticker customizado "CAMINHO[:posX:posY[:tamanho[:rotacao[:borda[:largura]]]]]"
/// carregando imagem do arquivo e gerando StickerElement com StickerKind::Custom.
fn parse_custom_sticker_spec(
    spec: &str,
    apply_palette: bool,
    default_border: StickerBorder,
    default_border_width: u32,
) -> Option<StickerElement> {
    let partes: Vec<&str> = spec.splitn(7, ':').collect();
    let img_path = partes.first().copied().unwrap_or("");
    let loaded = load_image(img_path).ok()?;
    let arc_img = std::sync::Arc::new(loaded);

    let element = if partes.len() >= 3 {
        let pos_x = partes[1].parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0);
        let pos_y = partes[2].parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0);
        let size = partes.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(100);
        let rotation = partes.get(4).and_then(|s| s.parse::<i16>().ok()).unwrap_or(0);
        let border = partes.get(5).and_then(|s| StickerBorder::parse(s)).unwrap_or(default_border);
        let border_width = partes.get(6).and_then(|s| s.parse::<u32>().ok()).unwrap_or(default_border_width);
        StickerElement::new_custom(arc_img, pos_x, pos_y, size, rotation, apply_palette)
            .with_border(border, border_width)
    } else {
        let idx = STICKER_AUTO_IDX.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v
        });
        let (px, py, rot) = STICKER_AUTO_POSITIONS[idx % STICKER_AUTO_POSITIONS.len()];
        StickerElement::new_custom(arc_img, px, py, 100, rot, apply_palette)
            .with_border(default_border, default_border_width)
    };
    Some(element)
}

fn load_image(input: &str) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {

    let bytes = if input == "-" {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        buffer
    } else {
        std::fs::read(input)?
    };

    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let dynamic = reader.decode()?;
    Ok(dynamic.to_rgba8())
}

fn resolve_palette(paleta_arg: &str) -> Palette {
    if let Some(p) = Palette::find_preset(paleta_arg) {
        p
    } else if let Some(custom) = Palette::parse_csv(paleta_arg) {
        custom
    } else {
        Palette::classic_punk()
    }
}

fn save_output(
    poster: &image::RgbaImage,
    comp: &PosterComposition,
    saida: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ext) = saida.extension().and_then(|s| s.to_str()) {
        if ext.eq_ignore_ascii_case("svg") {
            let svg_content = generate_poster_svg(comp, None);
            let mut f = File::create(saida)?;
            f.write_all(svg_content.as_bytes())?;
            return Ok(());
        }
    }

    let mut raw_png = Vec::new();
    poster.write_to(&mut Cursor::new(&mut raw_png), ImageFormat::Png)?;

    // Gera o comando CLI de reprodução fiel baseado nos argumentos fornecidos
    let args: Vec<String> = std::env::args().collect();
    let cmd_str = if !args.is_empty() {
        args.join(" ")
    } else {
        String::from("navalha poster")
    };

    let metadata = [
        ("Software", "Navalha CLI / Navalha Core"),
        ("Generator", "Navalha Motor Anti-Design"),
        ("Command", cmd_str.as_str()),
        ("License", "CC BY-SA 4.0"),
    ];

    let final_bytes = navalha_core::inject_png_metadata(&raw_png, &metadata)
        .unwrap_or(raw_png);

    let mut f = File::create(saida)?;
    f.write_all(&final_bytes)?;
    Ok(())
}

fn export_plates_to_dir(
    plates: &[PrintPlate],
    out_dir: &Path,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out_dir)?;

    let export_png = format == "png" || format == "ambos" || format == "both";
    let export_pdf = format == "pdf" || format == "ambos" || format == "both";

    if export_png {
        for plate in plates {
            let filename = format!(
                "chapa_{:02}_{}.png",
                plate.index,
                plate.color_hex.trim_start_matches('#')
            );
            let p = out_dir.join(filename);
            plate.image.save_with_format(&p, ImageFormat::Png)?;
            eprintln!(
                "   -> Chapa #{}: {} (cobertura: {:.1}%) salva em: {}",
                plate.index,
                plate.color_hex,
                plate.coverage_percentage,
                p.display()
            );
        }
    }

    if export_pdf {
        let pdf_data = generate_plates_pdf(plates, Some("Navalha - Chapas de Impressão (Risografia / Silk-screen)"))?;
        let pdf_path = out_dir.join("chapas.pdf");
        std::fs::write(&pdf_path, pdf_data)?;
        eprintln!(
            "   -> Documento PDF multichapas gerado em: {}",
            pdf_path.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, Rgba, RgbaImage};

    fn create_dummy_png(path: &Path, width: u32, height: u32) {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let v = ((x * 13 + y * 7) % 256) as u8;
                img.put_pixel(x, y, Rgba([v, 255 - v, 128, 255]));
            }
        }
        img.save_with_format(path, ImageFormat::Png).expect("Falha ao salvar dummy png");
    }

    #[test]
    fn test_extract_phrase_from_stem() {
        assert_eq!(extract_phrase_from_stem("contra-cultura_zine"), "CONTRA CULTURA ZINE");
        assert_eq!(extract_phrase_from_stem("fanzine-punk.01"), "FANZINE PUNK 01");
        assert_eq!(extract_phrase_from_stem("   "), "EU SOBREVIVO AO MUNDO");
    }

    #[test]
    fn test_collect_image_files_filtering_and_recursion() {
        let tmp_dir = std::env::temp_dir().join(format!("navalha_test_collect_{}", std::process::id()));
        let sub_dir = tmp_dir.join("subpasta");
        std::fs::create_dir_all(&sub_dir).unwrap();

        let f1 = tmp_dir.join("foto1.png");
        let f2 = tmp_dir.join("foto2.jpg");
        let f3 = tmp_dir.join("ignore.txt");
        let f4 = sub_dir.join("foto3.PNG");

        std::fs::write(&f1, b"fake").unwrap();
        std::fs::write(&f2, b"fake").unwrap();
        std::fs::write(&f3, b"fake").unwrap();
        std::fs::write(&f4, b"fake").unwrap();

        let exts = vec!["png".to_string(), "jpg".to_string()];

        // Não recursivo
        let mut non_rec = Vec::new();
        collect_image_files(&tmp_dir, &tmp_dir, false, &exts, &mut non_rec).unwrap();
        assert_eq!(non_rec.len(), 2);

        // Recursivo
        let mut rec = Vec::new();
        collect_image_files(&tmp_dir, &tmp_dir, true, &exts, &mut rec).unwrap();
        assert_eq!(rec.len(), 3);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_run_lote_poster_and_dither_modes() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let in_dir = std::env::temp_dir().join(format!("navalha_lote_in_{}", unique_id));
        let out_poster_dir = std::env::temp_dir().join(format!("navalha_lote_out_poster_{}", unique_id));
        let out_dither_dir = std::env::temp_dir().join(format!("navalha_lote_out_dither_{}", unique_id));

        std::fs::create_dir_all(&in_dir).unwrap();

        create_dummy_png(&in_dir.join("revolta-popular.png"), 60, 60);
        create_dummy_png(&in_dir.join("grito-rua.png"), 60, 60);

        // Teste 1: Modo Poster com frase do nome
        run_lote(
            in_dir.clone(),
            out_poster_dir.clone(),
            String::from("poster"),
            String::from("punk-riot"),
            String::from("FRASE BASE"),
            true, // frase_do_nome
            None,
            None,
            false,
            false,
            false,
            String::from("png,jpg"),
            String::from("atkinson"),
            String::from("classic-punk"),
            String::from("2x"),
            false,
            2,
        ).expect("Lote poster deve rodar com sucesso");

        let p1 = out_poster_dir.join("revolta-popular.png");
        let p2 = out_poster_dir.join("grito-rua.png");
        assert!(p1.exists(), "Pôster 1 deve ser gerado");
        assert!(p2.exists(), "Pôster 2 deve ser gerado");

        let img1 = image::open(&p1).unwrap();
        assert_eq!(img1.dimensions(), (800, 1100));

        // Teste 2: Modo Dither direto
        run_lote(
            in_dir.clone(),
            out_dither_dir.clone(),
            String::from("dither"),
            String::from("punk-riot"),
            String::from(""),
            false,
            None,
            None,
            true,
            false,
            false,
            String::from("png"),
            String::from("floyd-steinberg"),
            String::from("classic-punk"),
            String::from("2x"),
            false,
            1,
        ).expect("Lote dither deve rodar com sucesso");

        assert!(out_dither_dir.join("revolta-popular.png").exists());
        assert!(out_dither_dir.join("grito-rua.png").exists());

        // Limpeza
        let _ = std::fs::remove_dir_all(&in_dir);
        let _ = std::fs::remove_dir_all(&out_poster_dir);
        let _ = std::fs::remove_dir_all(&out_dither_dir);
    }

    #[test]
    fn test_run_lote_with_frases_arquivo() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let in_dir = std::env::temp_dir().join(format!("navalha_lote_frases_in_{}", unique_id));
        let out_dir = std::env::temp_dir().join(format!("navalha_lote_frases_out_{}", unique_id));
        let txt_path = in_dir.join("frases.txt");

        std::fs::create_dir_all(&in_dir).unwrap();
        create_dummy_png(&in_dir.join("fotoA.png"), 50, 50);
        create_dummy_png(&in_dir.join("fotoB.png"), 50, 50);

        std::fs::write(&txt_path, "PRIMEIRA LINHA\nSEGUNDA LINHA\n").unwrap();

        run_lote(
            in_dir.clone(),
            out_dir.clone(),
            String::from("poster"),
            String::from("punk-riot"),
            String::from("FALLBACK"),
            false,
            Some(txt_path),
            None,
            false,
            true, // ransom note
            false,
            String::from("png"),
            String::from("atkinson"),
            String::from("classic-punk"),
            String::from("2x"),
            false,
            2,
        ).expect("Lote com arquivo de frases deve rodar com sucesso");

        assert!(out_dir.join("fotoA.png").exists());
        assert!(out_dir.join("fotoB.png").exists());

        let _ = std::fs::remove_dir_all(&in_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
    }
}

