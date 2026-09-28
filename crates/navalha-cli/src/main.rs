#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::type_complexity
)]

use clap::{Parser, Subcommand};
use image::{ImageFormat, ImageReader};
use navalha_core::{
    generate_poster_svg, process_subject, render_poster, AestheticPreset,
    DamageElement, DitherAlgorithm, GuillotineConfig, Palette, PosterComposition, PrepConfig,
    PunkOverlayConfig, RgbColor, ScaleFactor, StickerBorder, StickerElement, StickerKind, TarjaBorderFinish, TarjaEffect, TextElement,
};
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};

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

        /// Estética anti-design: punk-riot, navalha-sata, xerox-podre, viagem-solitaria, lapa-noturna, teatro-operario, risografia, fanzine-digital, asfalto-alerta, vga256, cga, old-mac-pop-art, ascii-art, tabloide-popular, brutalismo, trash-polka, atari-gaming, mimeografo, xilogravura, glitch-crt
        #[arg(short, long, default_value = "punk-riot")]
        estetica: String,

        /// Rotação prévia da imagem em graus (0, 90, 180, 270)
        #[arg(long, default_value_t = 0)]
        rotacao: u32,

        /// Fator de zoom granular na foto (ex: 0.5 para afastar, 1.5 para aproximar, padrão 1.0)
        #[arg(long, default_value_t = 1.0)]
        zoom: f32,

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

        /// Arquivo de saída (.png ou .svg)
        #[arg(short = 'o', long = "saida", default_value = "poster_composto.png")]
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
            sem_faixa,
            sem_recorte,
            recorte,
            recorte_threshold,
            recorte_inverter,
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
            pr.rotation = rotacao;
            pr.zoom = zoom;
            comp.subject_offset_x = offset_x;
            comp.subject_offset_y = offset_y;

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

            if let Some(t) = comp.foreground_texts.get_mut(0) {
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
            dano,
            sticker,
            sticker_borda,
            sticker_borda_largura,
            sticker_custom,
            filtrar_stickers,
            texto_extra,
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

            if let Some(baixo) = tarja_baixo {
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

            let comp = PosterComposition {
                canvas_width: 800,
                canvas_height: 1100,
                background_color: RgbColor::new(245, 242, 235),
                background_text: bg_element,
                foreground_texts: fg_texts,
                overlay,
                stickers,
                palette: Some(palette.clone()),
                subject_offset_x: offset_x,
                subject_offset_y: offset_y,
            };

            let poster = navalha_core::render_poster(&comp, Some(&processed));
            save_output(&poster, &comp, &saida)?;
            eprintln!("[navalha] ⚡ Composição gravada em: {}", saida.display());
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
    }

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

    poster.save_with_format(saida, ImageFormat::Png)?;
    Ok(())
}
