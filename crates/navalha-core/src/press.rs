#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::manual_is_multiple_of
)]

use ab_glyph::{Font, FontRef, Glyph, Point, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use crate::palette::{Palette, RgbColor};
use serde::{Deserialize, Serialize};

// Fonte brutalista embutida no binário/wasm para determinismo absoluto em qualquer plataforma
pub const EMBEDDED_FONT_BYTES: &[u8] = include_bytes!("../assets/font.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TarjaEffect {
    #[default]
    Solida,
    Xerox,
}

impl TarjaEffect {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "solida" | "sólida" | "padrao" | "padrão" | "tarja" => Some(Self::Solida),
            "xerox" | "xerox-podre" | "fotocopia" | "fotocópia" => Some(Self::Xerox),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Solida => "solida",
            Self::Xerox => "xerox",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BannerFontFamily {
    #[default]
    Impact,
    Monospace,
    Serif,
    Sans,
}

impl BannerFontFamily {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "impact" | "bebas" | "poster" => Some(Self::Impact),
            "monospace" | "mono" | "terminal" => Some(Self::Monospace),
            "serif" | "jornal" | "romano" => Some(Self::Serif),
            "sans" | "helvetica" | "modern" => Some(Self::Sans),
            _ => None,
        }
    }

    pub fn css_font_family(&self) -> &'static str {
        match self {
            Self::Impact => "Impact, 'Bebas Neue', sans-serif",
            Self::Monospace => "'Courier New', Courier, monospace",
            Self::Serif => "'Times New Roman', Times, serif",
            Self::Sans => "Arial, Helvetica, sans-serif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TarjaBorderFinish {
    #[default]
    Nenhuma,
    Solida,
    Tracejada,
    Picote,
    FitaDurex,
}

impl TarjaBorderFinish {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "nenhuma" | "none" => Some(Self::Nenhuma),
            "solida" | "solid" => Some(Self::Solida),
            "tracejada" | "dashed" => Some(Self::Tracejada),
            "picote" | "serrilhado" | "dotted" => Some(Self::Picote),
            "durex" | "tape" | "fita" => Some(Self::FitaDurex),
            _ => None,
        }
    }
}

fn default_auto_fit() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    pub text: String,
    pub font_size: f32,
    pub stretch_x: f32,
    pub stretch_y: f32,
    pub color: RgbColor,
    pub bg_tarja: Option<RgbColor>,
    pub tarja_padding: u32,
    /// Posição normalizada (0.0 a 1.0) ou absoluta em pixels
    pub pos_x: f32,
    pub pos_y: f32,
    /// Se true, centraliza o texto no pos_x
    pub center_x: bool,
    /// Rotação simples em graus (0, 90, 180, 270)
    pub rotation: i16,
    #[serde(default)]
    pub tarja_effect: TarjaEffect,
    #[serde(default)]
    pub font_family: BannerFontFamily,
    #[serde(default = "default_auto_fit")]
    pub auto_fit: bool,
    #[serde(default)]
    pub border_width: u32,
    #[serde(default)]
    pub border_color: Option<RgbColor>,
    #[serde(default)]
    pub border_finish: TarjaBorderFinish,
}

impl Default for TextElement {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 48.0,
            stretch_x: 1.0,
            stretch_y: 1.4,
            color: RgbColor::new(255, 255, 255),
            bg_tarja: Some(RgbColor::new(0, 0, 0)),
            tarja_padding: 8,
            pos_x: 0.5,
            pos_y: 0.85,
            center_x: true,
            rotation: 0,
            tarja_effect: TarjaEffect::Solida,
            font_family: BannerFontFamily::Impact,
            auto_fit: true,
            border_width: 0,
            border_color: None,
            border_finish: TarjaBorderFinish::Nenhuma,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageEffect {
    Tiro,
    Rasgo,
    Chuva,
    Sangue,
    Tinta,
    Faca,
    Pixo,
    Censura,
    Pulica,
}

impl DamageEffect {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "tiro" => Some(Self::Tiro),
            "rasgo" => Some(Self::Rasgo),
            "chuva" => Some(Self::Chuva),
            "sangue" => Some(Self::Sangue),
            "tinta" => Some(Self::Tinta),
            "faca" => Some(Self::Faca),
            "pixo" => Some(Self::Pixo),
            "censura" => Some(Self::Censura),
            "pulica" | "puliça" => Some(Self::Pulica),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tiro => "tiro",
            Self::Rasgo => "rasgo",
            Self::Chuva => "chuva",
            Self::Sangue => "sangue",
            Self::Tinta => "tinta",
            Self::Faca => "faca",
            Self::Pixo => "pixo",
            Self::Censura => "censura",
            Self::Pulica => "pulica",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Tiro => "💥 Tiro",
            Self::Rasgo => "⚡ Rasgo",
            Self::Chuva => "🌧️ Chuva",
            Self::Sangue => "🩸 Sangue",
            Self::Tinta => "💧 Tinta",
            Self::Faca => "🗡️ Faca",
            Self::Pixo => "🔤 Pixo",
            Self::Censura => "⬛ Censura",
            Self::Pulica => "🚨 Puliça",
        }
    }

    pub fn all() -> &'static [DamageEffect] {
        &[
            DamageEffect::Tiro,
            DamageEffect::Rasgo,
            DamageEffect::Chuva,
            DamageEffect::Sangue,
            DamageEffect::Tinta,
            DamageEffect::Faca,
            DamageEffect::Pixo,
            DamageEffect::Censura,
            DamageEffect::Pulica,
        ]
    }
}

/// Instância customizável de um dano físico com posição e escala configuráveis.
///
/// Posição em valores normalizados (0.0 a 1.0) ou pixels absolutos se > 1.0.
/// Escala 1.0 equivale ao tamanho padrão de cada dano.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DamageElement {
    pub kind: DamageEffect,
    /// Posição horizontal do ponto focal (0.0–1.0 relativo ou px absoluto)
    pub pos_x: f32,
    /// Posição vertical do ponto focal (0.0–1.0 relativo ou px absoluto)
    pub pos_y: f32,
    /// Multiplicador de escala (1.0 = tamanho padrão)
    pub scale: f32,
}

impl DamageElement {
    pub fn new(kind: DamageEffect, pos_x: f32, pos_y: f32, scale: f32) -> Self {
        Self { kind, pos_x, pos_y, scale }
    }

    /// Cria um DamageElement com a posição e escala padrão para cada tipo de dano.
    pub fn default_for(kind: DamageEffect) -> Self {
        let (px, py) = match kind {
            DamageEffect::Tiro     => (0.50, 0.40),
            DamageEffect::Rasgo    => (0.80, 0.15),
            DamageEffect::Chuva    => (0.50, 0.50),
            DamageEffect::Sangue   => (0.45, 0.45),
            DamageEffect::Tinta    => (0.50, 0.05),
            DamageEffect::Faca     => (0.50, 0.35),
            DamageEffect::Pixo     => (0.20, 0.85),
            DamageEffect::Censura  => (0.50, 0.46),
            DamageEffect::Pulica   => (0.50, 0.72),
        };
        Self { kind, pos_x: px, pos_y: py, scale: 1.0 }
    }

    /// Faz o parsing de um especificador de string: "tipo:posX:posY:escala"
    /// ou apenas "tipo" (usa posição e escala padrão).
    pub fn parse_spec(spec: &str) -> Option<Self> {
        let partes: Vec<&str> = spec.splitn(4, ':').collect();
        let kind = DamageEffect::parse(partes.first().copied().unwrap_or(""))?;
        if partes.len() < 4 {
            return Some(Self::default_for(kind));
        }
        let pos_x = partes[1].parse::<f32>().unwrap_or_else(|_| Self::default_for(kind).pos_x);
        let pos_y = partes[2].parse::<f32>().unwrap_or_else(|_| Self::default_for(kind).pos_y);
        let scale = partes[3].parse::<f32>().unwrap_or(1.0).clamp(0.1, 5.0);
        Some(Self { kind, pos_x, pos_y, scale })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunkOverlayConfig {
    /// Moldura de corte nas bordas do pôster
    pub crop_frame: bool,
    /// Riscos manuais / arranhões de lâmina
    pub blade_scratches: bool,
    /// Marcas de registro / cruzes nas quinas
    pub registration_marks: bool,
    /// Cor dos vetores de intervenção
    pub vector_color: RgbColor,
    /// Lista de danos e intervenções físicas (modo simples legado, sem posição customizada)
    pub damages: Vec<DamageEffect>,
    /// Lista de danos com posição e escala customizáveis (sobrepõe `damages` se não vazia)
    pub damage_elements: Vec<DamageElement>,
}

impl Default for PunkOverlayConfig {
    fn default() -> Self {
        Self {
            crop_frame: true,
            blade_scratches: true,
            registration_marks: true,
            vector_color: RgbColor::new(255, 0, 51),
            damages: Vec::new(),
            damage_elements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StickerKind {
    Anarchy,
    RazorBlade,
    Skull,
    Biohazard,
    Copyleft,
    Barcode,
    SafetyPin,
    SmileyAcid,
    Custom,
}

impl StickerKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "anarchy" | "anarquia" => Some(Self::Anarchy),
            "razor" | "lamina" | "lâmina" | "razorblade" => Some(Self::RazorBlade),
            "skull" | "caveira" => Some(Self::Skull),
            "biohazard" | "radioativo" => Some(Self::Biohazard),
            "copyleft" | "anticopyright" | "anti-copyright" => Some(Self::Copyleft),
            "barcode" | "codigobarras" | "codigo-barras" => Some(Self::Barcode),
            "safetypin" | "alfinete" => Some(Self::SafetyPin),
            "smiley" | "acid" | "acid-smiley" => Some(Self::SmileyAcid),
            "custom" | "personalizado" | "proprio" | "recorte-custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anarchy => "anarchy",
            Self::RazorBlade => "razor",
            Self::Skull => "skull",
            Self::Biohazard => "biohazard",
            Self::Copyleft => "copyleft",
            Self::Barcode => "barcode",
            Self::SafetyPin => "safetypin",
            Self::SmileyAcid => "smiley",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Anarchy => "Ⓐ Anarquia",
            Self::RazorBlade => "🗡️ Lâmina",
            Self::Skull => "☠️ Caveira",
            Self::Biohazard => "☣️ Biohazard",
            Self::Copyleft => "🄯 Copyleft",
            Self::Barcode => "📊 Barcode",
            Self::SafetyPin => "🧷 Alfinete",
            Self::SmileyAcid => "☻ Acid Smile",
            Self::Custom => "★ Recorte Próprio",
        }
    }

    pub fn all() -> &'static [StickerKind] {
        &[
            StickerKind::Anarchy,
            StickerKind::RazorBlade,
            StickerKind::Skull,
            StickerKind::Biohazard,
            StickerKind::Copyleft,
            StickerKind::Barcode,
            StickerKind::SafetyPin,
            StickerKind::SmileyAcid,
            StickerKind::Custom,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StickerBorder {
    #[default]
    Nenhuma,
    DieCutBranco,
    Picote,
    Serrilhada,
    LinhaPreta,
}

impl StickerBorder {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "nenhuma" | "none" => Some(Self::Nenhuma),
            "diecut" | "die-cut" | "branco" | "diecutbranco" => Some(Self::DieCutBranco),
            "picote" | "perfurado" => Some(Self::Picote),
            "serrilhada" | "zigzag" | "dente" => Some(Self::Serrilhada),
            "preta" | "linhapreta" | "linha-preta" | "black" => Some(Self::LinhaPreta),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nenhuma => "nenhuma",
            Self::DieCutBranco => "diecut",
            Self::Picote => "picote",
            Self::Serrilhada => "serrilhada",
            Self::LinhaPreta => "preta",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Nenhuma => "Nenhuma",
            Self::DieCutBranco => "Die-Cut Branco (Adesivo)",
            Self::Picote => "Picotado Perfurado",
            Self::Serrilhada => "Serrilhado Dente-de-Serra",
            Self::LinhaPreta => "Linha Preta Sólida",
        }
    }

    pub fn all() -> &'static [StickerBorder] {
        &[
            Self::Nenhuma,
            Self::DieCutBranco,
            Self::Picote,
            Self::Serrilhada,
            Self::LinhaPreta,
        ]
    }
}

fn default_sticker_border_width() -> u32 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StickerElement {
    pub kind: StickerKind,
    pub pos_x: f32,
    pub pos_y: f32,
    pub size: u32,
    pub rotation: i16,
    pub apply_palette: bool,
    #[serde(default)]
    pub border: StickerBorder,
    #[serde(default = "default_sticker_border_width")]
    pub border_width: u32,
    #[serde(skip)]
    pub custom_image: Option<std::sync::Arc<RgbaImage>>,
}

impl StickerElement {
    pub fn new(
        kind: StickerKind,
        pos_x: f32,
        pos_y: f32,
        size: u32,
        rotation: i16,
        apply_palette: bool,
    ) -> Self {
        Self {
            kind,
            pos_x,
            pos_y,
            size,
            rotation,
            apply_palette,
            border: StickerBorder::Nenhuma,
            border_width: 4,
            custom_image: None,
        }
    }

    pub fn new_custom(
        image: std::sync::Arc<RgbaImage>,
        pos_x: f32,
        pos_y: f32,
        size: u32,
        rotation: i16,
        apply_palette: bool,
    ) -> Self {
        Self {
            kind: StickerKind::Custom,
            pos_x,
            pos_y,
            size,
            rotation,
            apply_palette,
            border: StickerBorder::Nenhuma,
            border_width: 4,
            custom_image: Some(image),
        }
    }

    pub fn with_custom_image(mut self, image: std::sync::Arc<RgbaImage>) -> Self {
        self.custom_image = Some(image);
        self.kind = StickerKind::Custom;
        self
    }

    pub fn with_border(mut self, border: StickerBorder, border_width: u32) -> Self {
        self.border = border;
        self.border_width = border_width;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosterComposition {
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// Camada 0: Cor de fundo sólida
    pub background_color: RgbColor,
    /// Camada 1: Texto maciço de fundo (sangrado)
    pub background_text: Option<TextElement>,
    /// Camada 3: Textos em tarja sobrepostos na frente
    pub foreground_texts: Vec<TextElement>,
    /// Camada 4: Riscos e molduras punk
    pub overlay: PunkOverlayConfig,
    /// Camada 5: Stickers e adesivos
    pub stickers: Vec<StickerElement>,
    /// Paleta para filtragem opcional de stickers
    pub palette: Option<Palette>,
    /// Deslocamento horizontal do sujeito em pixels (padrão 0 = centralizado)
    #[serde(default)]
    pub subject_offset_x: i32,
    /// Deslocamento vertical do sujeito em pixels (padrão 0 = centralizado)
    #[serde(default)]
    pub subject_offset_y: i32,
}

impl Default for PosterComposition {
    fn default() -> Self {
        Self {
            canvas_width: 800,
            canvas_height: 1100,
            background_color: RgbColor::new(245, 242, 235),
            background_text: None,
            foreground_texts: Vec::new(),
            overlay: PunkOverlayConfig::default(),
            stickers: Vec::new(),
            palette: None,
            subject_offset_x: 0,
            subject_offset_y: 0,
        }
    }
}

pub fn render_poster_cancellable<F>(
    composition: &PosterComposition,
    processed_subject: Option<&RgbaImage>,
    is_cancelled: F,
) -> Option<RgbaImage>
where
    F: Fn() -> bool,
{
    if is_cancelled() {
        return None;
    }

    let w = composition.canvas_width;
    let h = composition.canvas_height;
    let mut canvas = RgbaImage::new(w, h);

    let font = FontRef::try_from_slice(EMBEDDED_FONT_BYTES)
        .expect("Fonte embutida corrompida ou inválida");

    if is_cancelled() {
        return None;
    }

    // ── Camada 0: Fundo sólido chapado ──────────────────────────────────────────
    let bg = composition.background_color;
    for pixel in canvas.pixels_mut() {
        *pixel = Rgba([bg.r, bg.g, bg.b, 255]);
    }

    if is_cancelled() {
        return None;
    }

    // ── Camada 1: Texto de fundo gigante sangrado ───────────────────────────────
    if let Some(ref bg_text) = composition.background_text {
        render_text_element(&mut canvas, &font, bg_text);
    }

    if is_cancelled() {
        return None;
    }

    // ── Camada 2: Sujeito processado (recorte com dither) ───────────────────────
    if let Some(subject) = processed_subject {
        blend_subject(&mut canvas, subject, composition.subject_offset_x, composition.subject_offset_y);
    }

    if is_cancelled() {
        return None;
    }

    // ── Camada 3: Textos em tarja sobrepostos ───────────────────────────────────
    for fg_text in &composition.foreground_texts {
        if is_cancelled() {
            return None;
        }
        render_text_element(&mut canvas, &font, fg_text);
    }

    if is_cancelled() {
        return None;
    }

    // ── Camada 4: Riscos, vetores manuais e moldura de corte ────────────────────
    render_punk_overlay(&mut canvas, &composition.overlay, &font);

    if is_cancelled() {
        return None;
    }

    // ── Camada 5: Stickers colados ──────────────────────────────────────────────
    for sticker in &composition.stickers {
        if is_cancelled() {
            return None;
        }
        let palette_ref = if sticker.apply_palette {
            composition.palette.as_ref()
        } else {
            None
        };
        render_and_blend_sticker(&mut canvas, sticker, palette_ref, &font);
    }

    if is_cancelled() {
        return None;
    }

    Some(canvas)
}

pub fn render_poster(
    composition: &PosterComposition,
    processed_subject: Option<&RgbaImage>,
) -> RgbaImage {
    render_poster_cancellable(composition, processed_subject, || false)
        .expect("render_poster sem cancelamento deve retornar pôster")
}

fn blend_subject(canvas: &mut RgbaImage, subject: &RgbaImage, offset_x: i32, offset_y: i32) {
    let (cw, ch) = canvas.dimensions();
    let (sw, sh) = subject.dimensions();

    // Centraliza o sujeito no canvas mantendo proporção ou aplicando deslocamento customizado
    let base_ox = (cw as i32 - sw as i32) / 2 + offset_x;
    let base_oy = (ch as i32 - sh as i32) / 2 + offset_y;

    for sy in 0..sh {
        for sx in 0..sw {
            let cx = base_ox + sx as i32;
            let cy = base_oy + sy as i32;

            if cx >= 0 && cx < cw as i32 && cy >= 0 && cy < ch as i32 {
                let sp = subject.get_pixel(sx, sy);
                if sp[3] > 0 {
                    // Se for opaco ou semi-opaco
                    if sp[3] == 255 {
                        canvas.put_pixel(cx as u32, cy as u32, *sp);
                    } else {
                        let cp = canvas.get_pixel(cx as u32, cy as u32);
                        let alpha = sp[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        let r = (sp[0] as f32 * alpha + cp[0] as f32 * inv_alpha).round() as u8;
                        let g = (sp[1] as f32 * alpha + cp[1] as f32 * inv_alpha).round() as u8;
                        let b = (sp[2] as f32 * alpha + cp[2] as f32 * inv_alpha).round() as u8;
                        canvas.put_pixel(cx as u32, cy as u32, Rgba([r, g, b, 255]));
                    }
                }
            }
        }
    }
}

fn render_text_element(canvas: &mut RgbaImage, font: &FontRef, elem: &TextElement) {
    if elem.text.trim().is_empty() {
        return;
    }
    let (cw, ch) = canvas.dimensions();
    let (fam_stretch_x, fam_stretch_y, is_mono) = match elem.font_family {
        BannerFontFamily::Impact => (elem.stretch_x * 1.0, elem.stretch_y * 1.0, false),
        BannerFontFamily::Monospace => (elem.stretch_x * 0.92, elem.stretch_y * 0.88, true),
        BannerFontFamily::Serif => (elem.stretch_x * 1.05, elem.stretch_y * 0.92, false),
        BannerFontFamily::Sans => (elem.stretch_x * 0.94, elem.stretch_y * 0.95, false),
    };

    let mut scale = PxScale {
        x: elem.font_size * fam_stretch_x,
        y: elem.font_size * fam_stretch_y,
    };

    let raw_lines: Vec<&str> = elem.text.lines().collect();
    let lines = if raw_lines.is_empty() {
        vec![elem.text.as_str()]
    } else {
        raw_lines
    };

    // Auto-fit: mede a largura máxima e reduz proporcionalmente se exceder a largura disponível
    let max_allowed_width = (cw as f32 - 40.0 - (elem.tarja_padding as f32 * 2.0)).max(80.0);
    if elem.auto_fit {
        let test_font = font.as_scaled(scale);
        let mut test_max_w = 0.0f32;
        let mono_adv = test_font.height() * 0.58;
        for line in &lines {
            let mut line_w = 0.0f32;
            for ch in line.chars() {
                let adv = if is_mono {
                    mono_adv
                } else {
                    test_font.h_advance(font.glyph_id(ch))
                };
                line_w += adv;
            }
            if line_w > test_max_w {
                test_max_w = line_w;
            }
        }
        if test_max_w > max_allowed_width && test_max_w > 0.0 {
            let factor = max_allowed_width / test_max_w;
            scale.x *= factor;
            scale.y *= factor;
        }
    }

    let scaled_font = font.as_scaled(scale);
    let v_metrics = scaled_font.height();
    let line_height = v_metrics * 1.05;
    let mut line_records: Vec<(Vec<Glyph>, f32)> = Vec::with_capacity(lines.len());
    let mut max_line_width = 0.0f32;

    let mono_adv = scaled_font.height() * 0.58;
    for line in &lines {
        let mut glyphs: Vec<Glyph> = Vec::new();
        let mut cursor_x = 0.0f32;
        for ch in line.chars() {
            let glyph_id = font.glyph_id(ch);
            let advance = if is_mono {
                mono_adv
            } else {
                scaled_font.h_advance(glyph_id)
            };
            let glyph = glyph_id.with_scale_and_position(scale, Point { x: cursor_x, y: 0.0 });
            glyphs.push(glyph);
            cursor_x += advance;
        }
        if cursor_x > max_line_width {
            max_line_width = cursor_x;
        }
        line_records.push((glyphs, cursor_x));
    }

    let text_width = max_line_width;
    let text_height = v_metrics + (lines.len().saturating_sub(1) as f32) * line_height;

    // Converte pos_x e pos_y
    let base_x = if elem.pos_x <= 1.0 {
        elem.pos_x * cw as f32
    } else {
        elem.pos_x
    };

    let base_y = if elem.pos_y <= 1.0 {
        elem.pos_y * ch as f32
    } else {
        elem.pos_y
    };

    let start_x = if elem.center_x {
        base_x - (text_width / 2.0)
    } else {
        base_x
    };
    let start_y = base_y;

    if elem.rotation != 0 {
        let pad = elem.tarja_padding as f32;
        let sub_w = (text_width + pad * 2.0).ceil().max(1.0) as u32;
        let sub_h = (text_height + pad * 2.0).ceil().max(1.0) as u32;
        let mut sub_canvas = RgbaImage::new(sub_w, sub_h);

        if let Some(tarja_color) = elem.bg_tarja {
            match elem.tarja_effect {
                TarjaEffect::Solida => {
                    for ty in 0..sub_h {
                        for tx in 0..sub_w {
                            sub_canvas.put_pixel(tx, ty, Rgba([tarja_color.r, tarja_color.g, tarja_color.b, 255]));
                        }
                    }
                }
                TarjaEffect::Xerox => {
                    render_tarja_xerox(&mut sub_canvas, 0, 0, sub_w, sub_h, tarja_color);
                }
            }

            let border_finish = if elem.border_finish == TarjaBorderFinish::Nenhuma && elem.border_width > 0 {
                TarjaBorderFinish::Solida
            } else {
                elem.border_finish
            };
            let border_col = elem.border_color.unwrap_or(elem.color);
            let border_w = if border_finish != TarjaBorderFinish::Nenhuma && elem.border_width == 0 {
                3
            } else {
                elem.border_width
            };
            draw_tarja_border(&mut sub_canvas, 0, 0, sub_w, sub_h, border_finish, border_w, border_col);
        }

        // Rasteriza cada glifo em sua respectiva linha no sub_canvas
        for (line_idx, (glyphs, line_width)) in line_records.into_iter().enumerate() {
            let line_start_x = if elem.center_x {
                (sub_w as f32 - line_width) / 2.0
            } else {
                pad
            };
            let line_start_y = pad + (line_idx as f32) * line_height;

            for mut glyph in glyphs {
                glyph.position.x += line_start_x;
                glyph.position.y += line_start_y + scaled_font.ascent();

                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    outlined.draw(|x, y, c| {
                        if c > 0.15 {
                            let px = bounds.min.x as i32 + x as i32;
                            let py = bounds.min.y as i32 + y as i32;
                            if px >= 0 && px < sub_w as i32 && py >= 0 && py < sub_h as i32 {
                                if elem.tarja_effect == TarjaEffect::Xerox {
                                    if lcg_hash(px as u32, py as u32, 919) % 1000 < 30 {
                                        return;
                                    }
                                }
                                let col = elem.color;
                                sub_canvas.put_pixel(px as u32, py as u32, Rgba([col.r, col.g, col.b, 255]));
                            }
                        }
                    });
                }
            }
        }

        // Blit com rotação inversa em torno do ponto central da faixa
        let cx_target = if elem.center_x {
            base_x
        } else {
            base_x + text_width / 2.0
        };
        let cy_target = base_y + text_height / 2.0;

        let rad = (elem.rotation as f32).to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        let half_w = sub_w as f32 / 2.0;
        let half_h = sub_h as f32 / 2.0;
        let radius = (half_w * half_w + half_h * half_h).sqrt();

        let x0 = (cx_target - radius).max(0.0) as i32;
        let x1 = (cx_target + radius).min(cw as f32 - 1.0) as i32;
        let y0 = (cy_target - radius).max(0.0) as i32;
        let y1 = (cy_target + radius).min(ch as f32 - 1.0) as i32;

        for py in y0..=y1 {
            for px in x0..=x1 {
                let dx = px as f32 - cx_target;
                let dy = py as f32 - cy_target;
                let sx_f = half_w + (dx * cos_a + dy * sin_a);
                let sy_f = half_h + (-dx * sin_a + dy * cos_a);

                let sx = sx_f.round() as i32;
                let sy = sy_f.round() as i32;

                if sx >= 0 && sx < sub_w as i32 && sy >= 0 && sy < sub_h as i32 {
                    let sp = sub_canvas.get_pixel(sx as u32, sy as u32);
                    if sp[3] > 0 {
                        if sp[3] == 255 {
                            canvas.put_pixel(px as u32, py as u32, *sp);
                        } else {
                            let cp = canvas.get_pixel(px as u32, py as u32);
                            let alpha = sp[3] as f32 / 255.0;
                            let inv_alpha = 1.0 - alpha;
                            let r = (sp[0] as f32 * alpha + cp[0] as f32 * inv_alpha).round() as u8;
                            let g = (sp[1] as f32 * alpha + cp[1] as f32 * inv_alpha).round() as u8;
                            let b = (sp[2] as f32 * alpha + cp[2] as f32 * inv_alpha).round() as u8;
                            canvas.put_pixel(px as u32, py as u32, Rgba([r, g, b, 255]));
                        }
                    }
                }
            }
        }
        return;
    }

    // Desenha tarja de fundo se configurada
    if let Some(tarja_color) = elem.bg_tarja {
        let pad = elem.tarja_padding as f32;
        let x0 = (start_x - pad).max(0.0) as u32;
        let y0 = (start_y - pad).max(0.0) as u32;
        let x1 = (start_x + text_width + pad).min(cw as f32) as u32;
        let y1 = (start_y + text_height + pad).min(ch as f32) as u32;

        match elem.tarja_effect {
            TarjaEffect::Solida => {
                for ty in y0..y1 {
                    for tx in x0..x1 {
                        canvas.put_pixel(tx, ty, Rgba([tarja_color.r, tarja_color.g, tarja_color.b, 255]));
                    }
                }
            }
            TarjaEffect::Xerox => {
                render_tarja_xerox(canvas, x0, y0, x1, y1, tarja_color);
            }
        }

        let border_finish = if elem.border_finish == TarjaBorderFinish::Nenhuma && elem.border_width > 0 {
            TarjaBorderFinish::Solida
        } else {
            elem.border_finish
        };
        let border_col = elem.border_color.unwrap_or(elem.color);
        let border_w = if border_finish != TarjaBorderFinish::Nenhuma && elem.border_width == 0 {
            3
        } else {
            elem.border_width
        };
        draw_tarja_border(canvas, x0, y0, x1, y1, border_finish, border_w, border_col);
    }

    // Rasteriza cada glifo em sua respectiva linha
    for (line_idx, (glyphs, line_width)) in line_records.into_iter().enumerate() {
        let line_start_x = if elem.center_x {
            base_x - (line_width / 2.0)
        } else {
            base_x
        };
        let line_start_y = start_y + (line_idx as f32) * line_height;

        for mut glyph in glyphs {
            glyph.position.x += line_start_x;
            glyph.position.y += line_start_y + scaled_font.ascent();

            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|x, y, c| {
                    if c > 0.15 {
                        let px = bounds.min.x as i32 + x as i32;
                        let py = bounds.min.y as i32 + y as i32;
                        if px >= 0 && px < cw as i32 && py >= 0 && py < ch as i32 {
                            if elem.tarja_effect == TarjaEffect::Xerox {
                                if lcg_hash(px as u32, py as u32, 919) % 1000 < 30 {
                                    return;
                                }
                            }
                            let col = elem.color;
                            canvas.put_pixel(px as u32, py as u32, Rgba([col.r, col.g, col.b, 255]));
                        }
                    }
                });
            }
        }
    }
}

pub fn draw_tarja_border(
    canvas: &mut RgbaImage,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    finish: TarjaBorderFinish,
    width: u32,
    color: RgbColor,
) {
    if finish == TarjaBorderFinish::Nenhuma || width == 0 || x1 <= x0 || y1 <= y0 {
        return;
    }
    let (cw, ch) = canvas.dimensions();
    let border_rgba = Rgba([color.r, color.g, color.b, 255]);

    match finish {
        TarjaBorderFinish::Nenhuma => {}
        TarjaBorderFinish::Solida => {
            let bw = width.min((x1 - x0) / 2).min((y1 - y0) / 2).max(1);
            for x in x0..x1 {
                for y in y0..(y0 + bw).min(y1) {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
                for y in (y1.saturating_sub(bw)).max(y0)..y1 {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
            }
            for y in y0..y1 {
                for x in x0..(x0 + bw).min(x1) {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
                for x in (x1.saturating_sub(bw)).max(x0)..x1 {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
            }
        }
        TarjaBorderFinish::Tracejada => {
            let bw = width.min((x1 - x0) / 2).min((y1 - y0) / 2).max(1);
            for x in x0..x1 {
                if ((x - x0) / 12) % 2 == 0 {
                    for y in y0..(y0 + bw).min(y1) {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                    for y in (y1.saturating_sub(bw)).max(y0)..y1 {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                }
            }
            for y in y0..y1 {
                if ((y - y0) / 12) % 2 == 0 {
                    for x in x0..(x0 + bw).min(x1) {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                    for x in (x1.saturating_sub(bw)).max(x0)..x1 {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                }
            }
        }
        TarjaBorderFinish::Picote => {
            let bw = width.min((x1 - x0) / 2).min((y1 - y0) / 2).max(1);
            for x in x0..x1 {
                if ((x - x0) / 6) % 2 == 0 {
                    for y in y0..(y0 + bw).min(y1) {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                    for y in (y1.saturating_sub(bw)).max(y0)..y1 {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                }
            }
            for y in y0..y1 {
                if ((y - y0) / 6) % 2 == 0 {
                    for x in x0..(x0 + bw).min(x1) {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                    for x in (x1.saturating_sub(bw)).max(x0)..x1 {
                        if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                    }
                }
            }
        }
        TarjaBorderFinish::FitaDurex => {
            let bw = width.min((x1 - x0) / 2).min((y1 - y0) / 2).max(1);
            for x in x0..x1 {
                for y in y0..(y0 + bw).min(y1) {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
                for y in (y1.saturating_sub(bw)).max(y0)..y1 {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
            }
            for y in y0..y1 {
                for x in x0..(x0 + bw).min(x1) {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
                for x in (x1.saturating_sub(bw)).max(x0)..x1 {
                    if x < cw && y < ch { canvas.put_pixel(x, y, border_rgba); }
                }
            }
            let tab_len = 16u32;
            for i in 0..tab_len {
                let col = Rgba([230, 230, 200, 255]);
                if x0 + i < cw && y0 + i < ch { canvas.put_pixel(x0 + i, y0 + i, col); }
                if x1.saturating_sub(i) < cw && y0 + i < ch { canvas.put_pixel(x1.saturating_sub(i), y0 + i, col); }
                if x0 + i < cw && y1.saturating_sub(i) < ch { canvas.put_pixel(x0 + i, y1.saturating_sub(i), col); }
                if x1.saturating_sub(i) < cw && y1.saturating_sub(i) < ch { canvas.put_pixel(x1.saturating_sub(i), y1.saturating_sub(i), col); }
            }
        }
    }
}

pub fn lcg_hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393)
        ^ y.wrapping_mul(668265263)
        ^ salt.wrapping_mul(1013904223);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn render_tarja_xerox(
    canvas: &mut RgbaImage,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    tarja_color: RgbColor,
) {
    let (cw, ch) = canvas.dimensions();
    let tarja_rgba = Rgba([tarja_color.r, tarja_color.g, tarja_color.b, 255]);
    let h = y1.saturating_sub(y0);
    if h == 0 || x1 <= x0 {
        return;
    }

    // 1. Linhas horizontais de risco de varredura (drum streaks / roller marks)
    let scan_lines = [
        y0 + (h as f32 * 0.22) as u32,
        y0 + (h as f32 * 0.51) as u32,
        y0 + (h as f32 * 0.76) as u32,
        y0 + (h as f32 * 0.89) as u32,
    ];

    // 2. Extensão para salpicos e poeira de toner periférica
    let margin_dust = 6u32;
    let y_start = y0.saturating_sub(margin_dust);
    let y_end = (y1 + margin_dust).min(ch);

    for ty in y_start..y_end {
        // Jitter de varredura ótica lateral para cada linha y (-4px a +4px)
        let jitter_l = ((lcg_hash(ty, 0, 101) % 9) as i32) - 4;
        let jitter_r = ((lcg_hash(ty, 0, 211) % 9) as i32) - 4;
        let cur_x0 = (x0 as i32 + jitter_l).clamp(0, cw as i32) as u32;
        let cur_x1 = (x1 as i32 + jitter_r).clamp(0, cw as i32) as u32;

        let row_is_scanline = scan_lines.contains(&ty);

        let x_start = cur_x0.saturating_sub(margin_dust);
        let x_end = (cur_x1 + margin_dust).min(cw);

        for tx in x_start..x_end {
            let inside = ty >= y0 && ty < y1 && tx >= cur_x0 && tx < cur_x1;

            if inside {
                // Riscos horizontais da fotocopiadora: papel do fundo transparece
                if row_is_scanline && lcg_hash(tx, ty, 313) % 100 < 75 {
                    continue;
                }

                // Linha de arrasto vertical do rolete
                let w = cur_x1.saturating_sub(cur_x0);
                let roller_x = cur_x0 + (w as f32 * 0.38) as u32;
                if (tx == roller_x || tx == roller_x + 1) && lcg_hash(tx, ty, 401) % 100 < 45 {
                    continue;
                }

                // Distância das bordas da tarja
                let dist_top = ty.saturating_sub(y0);
                let dist_bottom = y1.saturating_sub(ty + 1);
                let dist_left = tx.saturating_sub(cur_x0);
                let dist_right = cur_x1.saturating_sub(tx + 1);
                let edge_dist = dist_top.min(dist_bottom).min(dist_left).min(dist_right);

                // Falhas pontuais de toner (toner dropout)
                let h_val = lcg_hash(tx, ty, 521);
                if edge_dist < 4 {
                    // Erosão nas quinas e bordas
                    if h_val % 100 < 24 {
                        continue;
                    }
                } else if h_val % 1000 < 35 {
                    // Granulação interna
                    continue;
                }

                canvas.put_pixel(tx, ty, tarja_rgba);
            } else {
                // Fora da tarja: pequenos salpicos esparsos de pó de toner
                let h_dust = lcg_hash(tx, ty, 733);
                if h_dust % 1000 < 12 {
                    canvas.put_pixel(tx, ty, tarja_rgba);
                }
            }
        }
    }
}

fn render_punk_overlay(canvas: &mut RgbaImage, overlay: &PunkOverlayConfig, font: &FontRef) {
    let (cw, ch) = canvas.dimensions();
    let col = overlay.vector_color;
    let rgba = Rgba([col.r, col.g, col.b, 255]);

    // 1. Moldura de corte punk nas bordas
    if overlay.crop_frame {
        let margin = 24u32;
        // Bordas externas com traço sólido de 3px
        for x in margin..(cw.saturating_sub(margin)) {
            for t in 0..3 {
                canvas.put_pixel(x, margin + t, rgba);
                canvas.put_pixel(x, ch.saturating_sub(margin + 1 + t), rgba);
            }
        }
        for y in margin..(ch.saturating_sub(margin)) {
            for t in 0..3 {
                canvas.put_pixel(margin + t, y, rgba);
                canvas.put_pixel(cw.saturating_sub(margin + 1 + t), y, rgba);
            }
        }
    }

    // 2. Cruzes de corte e marcas de registro nas quinas
    if overlay.registration_marks {
        let cross_size = 18i32;
        let corners = [
            (12i32, 12i32),
            (cw as i32 - 12, 12i32),
            (12i32, ch as i32 - 12),
            (cw as i32 - 12, ch as i32 - 12),
        ];

        for (cx, cy) in corners {
            for d in -cross_size..=cross_size {
                let px = cx + d;
                let py = cy;
                if px >= 0 && px < cw as i32 && py >= 0 && py < ch as i32 {
                    canvas.put_pixel(px as u32, py as u32, rgba);
                }
                let px = cx;
                let py = cy + d;
                if px >= 0 && px < cw as i32 && py >= 0 && py < ch as i32 {
                    canvas.put_pixel(px as u32, py as u32, rgba);
                }
            }
        }
    }

    // 3. Riscos manuais / arranhões de lâmina
    if overlay.blade_scratches {
        // Traços diagonais toscos anti-design
        let scratch_lines = [
            (30i32, 60i32, 180i32, 85i32),
            (cw as i32 - 150, ch as i32 - 100, cw as i32 - 40, ch as i32 - 60),
            (cw as i32 / 2 - 80, ch as i32 - 45, cw as i32 / 2 + 80, ch as i32 - 42),
        ];

        for (x0, y0, x1, y1) in scratch_lines {
            draw_rough_line(canvas, x0, y0, x1, y1, rgba);
        }
    }

    // 4. Danos físicos e intervenções brutas.
    // Usa damage_elements (com posição/escala customizáveis) se preenchido;
    // senão converte o Vec legado para DamageElement com posições padrão.
    let elementos: Vec<DamageElement> = if !overlay.damage_elements.is_empty() {
        overlay.damage_elements.clone()
    } else {
        overlay.damages.iter()
            .map(|&d| DamageElement::default_for(d))
            .collect()
    };

    for elem in &elementos {
        render_damage_element(canvas, elem, font, cw, ch);
    }
}

fn draw_filled_circle(canvas: &mut RgbaImage, cx: i32, cy: i32, r: i32, col: Rgba<u8>) {
    let (cw, ch) = canvas.dimensions();
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < cw as i32 && py >= 0 && py < ch as i32 {
                    canvas.put_pixel(px as u32, py as u32, col);
                }
            }
        }
    }
}

fn draw_filled_rect(canvas: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, col: Rgba<u8>) {
    let (cw, ch) = canvas.dimensions();
    let min_x = x0.min(x1).max(0) as u32;
    let max_x = x0.max(x1).min(cw as i32 - 1) as u32;
    let min_y = y0.min(y1).max(0) as u32;
    let max_y = y0.max(y1).min(ch as i32 - 1) as u32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            canvas.put_pixel(x, y, col);
        }
    }
}

fn render_damage_tiro(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32) {
    let s = scale;
    let black = Rgba([5, 5, 5, 255]);
    let paper_rim = Rgba([240, 238, 230, 255]);
    let burn_rim = Rgba([50, 45, 45, 220]);

    let cracks = [
        (0, -38), (28, -28), (42, 0), (28, 30),
        (0, 40), (-32, 28), (-40, -4), (-25, -28)
    ];
    for (dx, dy) in cracks {
        draw_rough_line(canvas, cx, cy, cx + (dx as f32 * s) as i32, cy + (dy as f32 * s) as i32, burn_rim);
    }
    draw_filled_circle(canvas, cx, cy, (20.0 * s) as i32, burn_rim);
    draw_filled_circle(canvas, cx, cy, (14.0 * s) as i32, paper_rim);
    draw_filled_circle(canvas, cx, cy, (9.0 * s) as i32, black);
}

fn render_damage_rasgo(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32) {
    let pulp = Rgba([245, 242, 232, 255]);
    let shadow = Rgba([15, 15, 15, 230]);
    let len_x = (80.0 * scale) as i32;
    let len_y = (220.0 * scale) as i32;
    let start_x = cx - len_x / 2;
    let start_y = cy - len_y / 4;
    let end_x   = cx + len_x / 2;
    let end_y   = cy + len_y * 3 / 4;

    let steps = 40;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let bx = start_x as f32 + (end_x - start_x) as f32 * t;
        let by = start_y as f32 + (end_y - start_y) as f32 * t;
        let jitter = ((i * 7) % 11) as f32 - 5.0;
        let x = (bx + jitter) as i32;
        let y = (by + jitter) as i32;

        draw_filled_circle(canvas, x, y + 3, (5.0 * scale) as i32, shadow);
        draw_filled_circle(canvas, x, y, (4.0 * scale) as i32, pulp);
    }
}

fn render_damage_chuva(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32, cw: u32, ch: u32) {
    let mut seed = 19770816u32;
    let rain_col = Rgba([230, 230, 240, 180]);
    let area_w = (cw as f32 * scale).min(cw as f32) as u32;
    let area_h = (ch as f32 * scale).min(ch as f32) as u32;
    let ox = (cx - (area_w / 2) as i32).max(0) as u32;
    let oy = (cy - (area_h / 2) as i32).max(0) as u32;
    let drops = ((140.0 * scale) as u32).max(20);
    for _ in 0..drops {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let x = ox as i32 + (seed % area_w.max(1)) as i32;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let y = oy as i32 + (seed % area_h.max(1)) as i32;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let len = ((25.0 + (seed % 45) as f32) * scale) as i32;
        draw_rough_line(canvas, x, y, x - 1, (y + len).min(ch as i32 - 1), rain_col);
    }
}

fn render_damage_sangue(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32) {
    let s = scale;
    let blood = Rgba([140, 0, 10, 255]);
    let dark_blood = Rgba([90, 0, 0, 255]);

    draw_filled_circle(canvas, cx, cy, (22.0 * s) as i32, blood);
    draw_filled_circle(canvas, cx + (2.0 * s) as i32, cy - (2.0 * s) as i32, (16.0 * s) as i32, dark_blood);
    draw_rough_line(canvas, cx - (6.0*s) as i32, cy + (10.0*s) as i32, cx - (7.0*s) as i32, cy + (45.0*s) as i32, blood);
    draw_filled_circle(canvas, cx - (7.0*s) as i32, cy + (45.0*s) as i32, (5.0*s) as i32, blood);
    draw_rough_line(canvas, cx + (8.0*s) as i32, cy + (12.0*s) as i32, cx + (9.0*s) as i32, cy + (70.0*s) as i32, blood);
    draw_filled_circle(canvas, cx + (9.0*s) as i32, cy + (70.0*s) as i32, (6.0*s) as i32, blood);

    let satellites = [
        (-35, -20, 4), (30, -32, 3), (45, 15, 4), (-25, 40, 3),
        (-45, 5, 2), (25, 38, 3), (0, -42, 3)
    ];
    for (dx, dy, r) in satellites {
        draw_filled_circle(canvas, cx + (dx as f32 * s) as i32, cy + (dy as f32 * s) as i32, (r as f32 * s) as i32, blood);
    }
}

fn render_damage_tinta(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32, cw: u32) {
    let ink = Rgba([8, 8, 12, 255]);
    let s = scale;
    let puddle_half = (cw as f32 * 0.35 * s) as i32;
    let puddle_h = (28.0 * s) as i32;
    let puddle_y0 = cy;
    let puddle_y1 = cy + puddle_h;
    draw_filled_rect(canvas, cx - puddle_half, puddle_y0, cx + puddle_half, puddle_y1, ink);

    let drips = [
        (-puddle_half * 3 / 4, (110.0 * s) as i32, (5.0 * s) as i32),
        (-puddle_half / 4,     (180.0 * s) as i32, (6.0 * s) as i32),
        (0,                    (130.0 * s) as i32, (4.0 * s) as i32),
        (puddle_half / 4,      (220.0 * s) as i32, (7.0 * s) as i32),
        (puddle_half * 3 / 4,  (90.0  * s) as i32, (4.0 * s) as i32),
    ];
    for (dx, len, thick) in drips {
        for t in 0..thick {
            draw_rough_line(canvas, cx + dx + t, puddle_y1, cx + dx + t, puddle_y1 + len, ink);
        }
        draw_filled_circle(canvas, cx + dx + thick / 2, puddle_y1 + len, thick + (4.0 * s) as i32, ink);
    }
}

fn render_damage_faca(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32, cw: u32, ch: u32) {
    let cut = Rgba([10, 10, 10, 255]);
    let white = Rgba([255, 255, 255, 230]);
    let hw = (cw as f32 * 0.42 * scale) as i32;
    let hh1 = (ch as f32 * 0.10 * scale) as i32;
    let hh2 = (ch as f32 * 0.10 * scale) as i32;
    let oy2 = (ch as f32 * 0.16 * scale) as i32;
    draw_rough_line(canvas, cx - hw, cy - hh1, cx + hw, cy + hh1, cut);
    draw_rough_line(canvas, cx - hw, cy - hh1 + 2, cx + hw, cy + hh1 + 2, white);
    draw_rough_line(canvas, cx - hw, cy + oy2 - hh2, cx + hw, cy + oy2 + hh2, cut);
    draw_rough_line(canvas, cx - hw, cy + oy2 - hh2 + 2, cx + hw, cy + oy2 + hh2 + 2, white);
}

fn render_damage_pixo(canvas: &mut RgbaImage, cx: i32, cy: i32, scale: f32) {
    let pixo_col = Rgba([255, 255, 255, 255]);
    let shadow = Rgba([0, 0, 0, 255]);
    let s = |v: i32| -> i32 { (v as f32 * scale) as i32 };
    // bx, by são o canto superior esquerdo do bloco de grafite, relativo ao cx, cy
    let bx = cx - s(95);
    let by = cy - s(40);

    let strokes = [
        (bx,       by,       bx,       by+s(80)),
        (bx,       by,       bx+s(30), by+s(80)),
        (bx+s(30), by,       bx+s(30), by+s(80)),
        (bx+s(40), by,       bx+s(55), by+s(80)),
        (bx+s(55), by+s(80), bx+s(70), by),
        (bx+s(80), by,       bx+s(80), by+s(80)),
        (bx+s(80), by+s(80), bx+s(110),by+s(80)),
        (bx+s(120),by,       bx+s(120),by+s(80)),
        (bx+s(150),by,       bx+s(150),by+s(80)),
        (bx+s(120),by+s(40), bx+s(150),by+s(40)),
        (bx+s(165),by+s(50), bx+s(185),by+s(10)),
        (bx+s(170),by+s(10), bx+s(185),by+s(10)),
        (bx+s(185),by+s(10), bx+s(185),by+s(25)),
    ];

    for &(x0, y0, x1, y1) in &strokes {
        draw_rough_line(canvas, x0 + 2, y0 + 2, x1 + 2, y1 + 2, shadow);
    }
    for &(x0, y0, x1, y1) in &strokes {
        draw_rough_line(canvas, x0, y0, x1, y1, pixo_col);
    }
}

fn render_damage_censura(canvas: &mut RgbaImage, font: &FontRef, pos_x: f32, pos_y: f32, scale: f32) {
    let elem = TextElement {
        text: "[ CENSURADO ]".to_string(),
        font_size: 44.0 * scale,
        stretch_x: 1.2,
        stretch_y: 1.5,
        color: RgbColor::new(255, 255, 255),
        bg_tarja: Some(RgbColor::new(0, 0, 0)),
        tarja_padding: (16.0 * scale) as u32,
        pos_x,
        pos_y,
        center_x: true,
        rotation: 0,
        tarja_effect: TarjaEffect::Solida,
        ..Default::default()
    };
    render_text_element(canvas, font, &elem);
}

fn render_damage_pulica(canvas: &mut RgbaImage, font: &FontRef, pos_x: f32, pos_y: f32, scale: f32, cw: u32, ch: u32) {
    let elem = TextElement {
        text: "PULIÇA / APREENDIDO".to_string(),
        font_size: 32.0 * scale,
        stretch_x: 1.1,
        stretch_y: 1.4,
        color: RgbColor::new(220, 20, 30),
        bg_tarja: Some(RgbColor::new(245, 235, 235)),
        tarja_padding: (14.0 * scale) as u32,
        pos_x,
        pos_y,
        center_x: true,
        rotation: 0,
        tarja_effect: TarjaEffect::Solida,
        ..Default::default()
    };
    render_text_element(canvas, font, &elem);

    let box_cx = if pos_x <= 1.0 { (pos_x * cw as f32) as i32 } else { pos_x as i32 };
    let box_cy = if pos_y <= 1.0 { (pos_y * ch as f32) as i32 } else { pos_y as i32 };
    let half_w = (210.0 * scale) as i32;
    let box_x0 = box_cx - half_w;
    let box_x1 = box_cx + half_w;
    let box_y0 = box_cy - (20.0 * scale) as i32;
    let box_y1 = box_cy + (55.0 * scale) as i32;
    let stamp_col = Rgba([220, 20, 30, 230]);

    for x in box_x0..=box_x1 {
        if (x * 3) % 7 != 0 && x >= 0 && x < cw as i32 {
            if box_y0 >= 0 && box_y0 < ch as i32 {
                canvas.put_pixel(x as u32, box_y0 as u32, stamp_col);
            }
            if box_y1 >= 0 && box_y1 < ch as i32 {
                canvas.put_pixel(x as u32, box_y1 as u32, stamp_col);
            }
        }
    }
    for y in box_y0..=box_y1 {
        if (y * 5) % 8 != 0 && y >= 0 && y < ch as i32 {
            if box_x0 >= 0 && box_x0 < cw as i32 {
                canvas.put_pixel(box_x0 as u32, y as u32, stamp_col);
            }
            if box_x1 >= 0 && box_x1 < cw as i32 {
                canvas.put_pixel(box_x1 as u32, y as u32, stamp_col);
            }
        }
    }
}

/// Renderiza um DamageElement com posição e escala customizáveis.
///
/// Posição 0.0–1.0 = relativo ao canvas. Escala 1.0 = tamanho padrão.
fn render_damage_element(
    canvas: &mut RgbaImage,
    elem: &DamageElement,
    font: &FontRef,
    cw: u32,
    ch: u32,
) {
    let scale = elem.scale;
    let cx = if elem.pos_x <= 1.0 { (elem.pos_x * cw as f32) as i32 } else { elem.pos_x as i32 };
    let cy = if elem.pos_y <= 1.0 { (elem.pos_y * ch as f32) as i32 } else { elem.pos_y as i32 };

    match elem.kind {
        DamageEffect::Tiro    => render_damage_tiro(canvas, cx, cy, scale),
        DamageEffect::Rasgo   => render_damage_rasgo(canvas, cx, cy, scale),
        DamageEffect::Chuva   => render_damage_chuva(canvas, cx, cy, scale, cw, ch),
        DamageEffect::Sangue  => render_damage_sangue(canvas, cx, cy, scale),
        DamageEffect::Tinta   => render_damage_tinta(canvas, cx, cy, scale, cw),
        DamageEffect::Faca    => render_damage_faca(canvas, cx, cy, scale, cw, ch),
        DamageEffect::Pixo    => render_damage_pixo(canvas, cx, cy, scale),
        DamageEffect::Censura => render_damage_censura(canvas, font, elem.pos_x, elem.pos_y, scale),
        DamageEffect::Pulica  => render_damage_pulica(canvas, font, elem.pos_x, elem.pos_y, scale, cw, ch),
    }
}

fn draw_rough_line(canvas: &mut RgbaImage, mut x0: i32, mut y0: i32, x1: i32, y1: i32, col: Rgba<u8>) {
    let (cw, ch) = canvas.dimensions();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < cw as i32 && y0 >= 0 && y0 < ch as i32 {
            canvas.put_pixel(x0 as u32, y0 as u32, col);
            // Dobra espessura para visual brutalista de caneta
            if x0 + 1 < cw as i32 {
                canvas.put_pixel((x0 + 1) as u32, y0 as u32, col);
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_circle_outline(
    canvas: &mut RgbaImage,
    cx: i32,
    cy: i32,
    r: i32,
    thickness: i32,
    col: Rgba<u8>,
) {
    let (cw, ch) = canvas.dimensions();
    let r_outer = r + thickness;
    let r_inner = (r - thickness).max(0);
    for dy in -r_outer..=r_outer {
        for dx in -r_outer..=r_outer {
            let d2 = dx * dx + dy * dy;
            if d2 >= r_inner * r_inner && d2 <= r_outer * r_outer {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < cw as i32 && py >= 0 && py < ch as i32 {
                    canvas.put_pixel(px as u32, py as u32, col);
                }
            }
        }
    }
}

pub fn render_sticker(kind: StickerKind, size: u32, font: &FontRef) -> RgbaImage {
    match kind {
        StickerKind::Anarchy => render_sticker_anarchy(size),
        StickerKind::RazorBlade => render_sticker_razor(size, font),
        StickerKind::Skull => render_sticker_skull(size),
        StickerKind::Biohazard => render_sticker_biohazard(size),
        StickerKind::Copyleft => render_sticker_copyleft(size),
        StickerKind::Barcode => render_sticker_barcode(size),
        StickerKind::SafetyPin => render_sticker_safetypin(size),
        StickerKind::SmileyAcid => render_sticker_smiley(size),
        StickerKind::Custom => render_sticker_custom_placeholder(size),
    }
}

fn render_sticker_custom_placeholder(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r = (size as i32 / 2) - 4;
    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let gold = Rgba([255, 215, 0, 255]);

    draw_filled_circle(&mut img, cx, cy, r, black);
    draw_circle_outline(&mut img, cx, cy, r, 2, white);
    draw_filled_circle(&mut img, cx, cy, (r * 2) / 3, gold);
    draw_circle_outline(&mut img, cx, cy, (r * 2) / 3, 2, black);
    draw_rough_line(&mut img, cx - r / 2, cy, cx + r / 2, cy, white);
    draw_rough_line(&mut img, cx, cy - r / 2, cx, cy + r / 2, white);

    img
}

fn render_sticker_anarchy(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r_base = (size as i32 / 2) - 2;
    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([20, 20, 20, 255]);
    let red = Rgba([255, 0, 51, 255]);

    draw_filled_circle(&mut img, cx, cy, r_base, white);
    draw_filled_circle(&mut img, cx, cy, r_base - 3, black);

    let r_ring = (size as f32 * 0.35) as i32;
    draw_circle_outline(&mut img, cx, cy, r_ring, 3, red);

    let apex_y = cy - (r_ring as f32 * 1.35) as i32;
    let foot_y = cy + (r_ring as f32 * 1.25) as i32;
    let left_foot_x = cx - (r_ring as f32 * 1.1) as i32;
    let right_foot_x = cx + (r_ring as f32 * 1.1) as i32;
    let bar_y = cy + (r_ring as f32 * 0.15) as i32;

    draw_rough_line(&mut img, cx, apex_y, left_foot_x, foot_y, red);
    draw_rough_line(&mut img, cx, apex_y, right_foot_x, foot_y, red);
    draw_rough_line(&mut img, left_foot_x - 4, bar_y, right_foot_x + 4, bar_y, red);

    img
}

fn render_sticker_razor(size: u32, _font: &FontRef) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let bw = (size as f32 * 0.88) as i32;
    let bh = (size as f32 * 0.52) as i32;
    let x0 = cx - bw / 2;
    let x1 = cx + bw / 2;
    let y0 = cy - bh / 2;
    let y1 = cy + bh / 2;

    let white = Rgba([255, 255, 255, 255]);
    let silver = Rgba([215, 215, 225, 255]);
    let slot_col = Rgba([15, 15, 20, 255]);

    draw_filled_rect(&mut img, x0 - 2, y0 - 2, x1 + 2, y1 + 2, white);
    draw_filled_rect(&mut img, x0, y0, x1, y1, silver);

    draw_filled_rect(&mut img, x0 + 4, y0, x1 - 4, y0 + 3, white);
    draw_filled_rect(&mut img, x0 + 4, y1 - 3, x1 - 4, y1, white);

    for &(dx, dy) in &[(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
        draw_filled_rect(&mut img, dx - 3, dy - 3, dx + 3, dy + 3, Rgba([0, 0, 0, 0]));
    }

    draw_filled_rect(&mut img, cx - bw / 4, cy - 3, cx + bw / 4, cy + 3, slot_col);
    draw_filled_circle(&mut img, cx, cy, 6, slot_col);
    draw_filled_circle(&mut img, cx - bw / 4, cy, 5, slot_col);
    draw_filled_circle(&mut img, cx + bw / 4, cy, 5, slot_col);

    img
}

fn render_sticker_skull(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r_base = (size as i32 / 2) - 2;

    let white = Rgba([255, 255, 255, 255]);
    let dark = Rgba([15, 15, 15, 255]);
    let bone_col = Rgba([240, 240, 235, 255]);

    draw_filled_circle(&mut img, cx, cy, r_base, white);
    draw_filled_circle(&mut img, cx, cy, r_base - 3, dark);

    draw_rough_line(&mut img, cx - 26, cy - 26, cx + 26, cy + 26, bone_col);
    draw_rough_line(&mut img, cx - 26, cy + 26, cx + 26, cy - 26, bone_col);

    draw_filled_circle(&mut img, cx, cy - 4, (size as f32 * 0.26) as i32, bone_col);
    draw_filled_rect(&mut img, cx - 11, cy + 4, cx + 11, cy + 20, bone_col);

    draw_filled_circle(&mut img, cx - 8, cy - 2, 5, dark);
    draw_filled_circle(&mut img, cx + 8, cy - 2, 5, dark);
    draw_filled_circle(&mut img, cx, cy + 6, 3, dark);

    for t in [-7, -2, 3, 8] {
        draw_rough_line(&mut img, cx + t, cy + 13, cx + t, cy + 20, dark);
    }

    img
}

fn render_sticker_biohazard(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r_base = (size as i32 / 2) - 2;
    let yellow = Rgba([255, 230, 0, 255]);
    let black = Rgba([0, 0, 0, 255]);

    draw_filled_circle(&mut img, cx, cy, r_base, Rgba([255, 255, 255, 255]));
    draw_filled_circle(&mut img, cx, cy, r_base - 3, yellow);
    draw_circle_outline(&mut img, cx, cy, r_base - 8, 3, black);

    let r_lobe = (size as f32 * 0.22) as i32;
    let dist = (size as f32 * 0.18) as i32;
    let angles = [90.0f32, 210.0f32, 330.0f32];
    for a in angles {
        let rad = a.to_radians();
        let lx = cx + (dist as f32 * rad.cos()) as i32;
        let ly = cy + (dist as f32 * rad.sin()) as i32;
        draw_circle_outline(&mut img, lx, ly, r_lobe, 3, black);
    }
    draw_circle_outline(&mut img, cx, cy, (size as f32 * 0.12) as i32, 2, black);
    draw_filled_circle(&mut img, cx, cy, 4, black);

    img
}

fn render_sticker_copyleft(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r = (size as i32 / 2) - 3;
    let white = Rgba([255, 255, 255, 255]);

    draw_filled_circle(&mut img, cx, cy, r, white);
    draw_filled_circle(&mut img, cx, cy, r - 3, Rgba([10, 10, 10, 255]));
    draw_circle_outline(&mut img, cx, cy, (size as f32 * 0.32) as i32, 3, white);

    let r_c = (size as f32 * 0.22) as i32;
    for dy in -r_c..=r_c {
        for dx in -r_c..=r_c {
            let d2 = dx * dx + dy * dy;
            if d2 >= (r_c - 4) * (r_c - 4) && d2 <= r_c * r_c {
                if dx >= -(r_c / 3) {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && px < size as i32 && py >= 0 && py < size as i32 {
                        img.put_pixel(px as u32, py as u32, white);
                    }
                }
            }
        }
    }

    img
}

fn render_sticker_barcode(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let w = (size as f32 * 0.9) as i32;
    let h = (size as f32 * 0.58) as i32;
    let x0 = cx - w / 2;
    let x1 = cx + w / 2;
    let y0 = cy - h / 2;
    let y1 = cy + h / 2;

    let white = Rgba([255, 255, 255, 255]);
    let paper = Rgba([245, 245, 245, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let red = Rgba([255, 0, 51, 255]);

    draw_filled_rect(&mut img, x0 - 2, y0 - 2, x1 + 2, y1 + 2, white);
    draw_filled_rect(&mut img, x0, y0, x1, y1, paper);

    let mut bx = x0 + 6;
    let pattern = [2, 1, 3, 1, 2, 4, 1, 2, 1, 3, 2, 1, 4, 1, 2, 3, 1, 2];
    for (i, &p) in pattern.iter().enumerate() {
        if i % 2 == 0 {
            for t in 0..p {
                if bx + t < x1 - 6 {
                    draw_rough_line(&mut img, bx + t, y0 + 6, bx + t, y1 - 10, black);
                }
            }
        }
        bx += p + 2;
    }

    draw_rough_line(&mut img, x0 + 4, y1 - 6, x1 - 4, y0 + 6, red);
    draw_rough_line(&mut img, x0 + 4, y1 - 4, x1 - 4, y0 + 8, red);

    img
}

fn render_sticker_safetypin(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let silver = Rgba([215, 215, 225, 255]);
    let white = Rgba([255, 255, 255, 255]);
    let shadow = Rgba([40, 40, 45, 255]);

    draw_filled_rect(&mut img, cx - 18, cy - 28, cx + 6, cy - 14, white);
    draw_filled_rect(&mut img, cx - 16, cy - 26, cx + 4, cy - 16, silver);

    draw_rough_line(&mut img, cx - 12, cy - 14, cx - 12, cy + 24, silver);
    draw_rough_line(&mut img, cx, cy - 14, cx, cy + 24, shadow);
    draw_rough_line(&mut img, cx - 1, cy - 14, cx - 1, cy + 24, silver);

    draw_circle_outline(&mut img, cx - 6, cy + 24, 7, 2, silver);

    img
}

fn render_sticker_smiley(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cx = size as i32 / 2;
    let cy = size as i32 / 2;
    let r = (size as i32 / 2) - 2;
    let white = Rgba([255, 255, 255, 255]);
    let acid_yellow = Rgba([210, 255, 0, 255]);
    let black = Rgba([0, 0, 0, 255]);

    draw_filled_circle(&mut img, cx, cy, r, white);
    draw_filled_circle(&mut img, cx, cy, r - 3, acid_yellow);
    draw_circle_outline(&mut img, cx, cy, r - 4, 3, black);

    draw_rough_line(&mut img, cx - 14, cy - 14, cx - 6, cy - 6, black);
    draw_rough_line(&mut img, cx - 6, cy - 14, cx - 14, cy - 6, black);
    draw_rough_line(&mut img, cx + 6, cy - 14, cx + 14, cy - 6, black);
    draw_rough_line(&mut img, cx + 14, cy - 14, cx + 6, cy - 6, black);

    let pts = [
        (cx - 16, cy + 12),
        (cx - 10, cy + 16),
        (cx - 4, cy + 12),
        (cx + 2, cy + 16),
        (cx + 8, cy + 12),
        (cx + 14, cy + 16),
    ];
    for i in 0..pts.len() - 1 {
        draw_rough_line(&mut img, pts[i].0, pts[i].1, pts[i + 1].0, pts[i + 1].1, black);
    }

    img
}

pub fn apply_sticker_border(
    src: &RgbaImage,
    border: StickerBorder,
    border_width: u32,
) -> RgbaImage {
    if border == StickerBorder::Nenhuma || border_width == 0 {
        return src.clone();
    }

    let (sw, sh) = src.dimensions();
    let pad = (border_width + 2) as i32;
    let nw = sw + (pad as u32 * 2);
    let nh = sh + (pad as u32 * 2);
    let mut out = RgbaImage::new(nw, nh);

    let bw_f = border_width as f32;
    let bw2 = bw_f * bw_f;

    let mut is_opaque = vec![false; (sw * sh) as usize];
    for y in 0..sh {
        for x in 0..sw {
            if src.get_pixel(x, y)[3] > 64 {
                is_opaque[(y * sw + x) as usize] = true;
            }
        }
    }

    for oy in 0..nh {
        for ox in 0..nw {
            let sx = ox as i32 - pad;
            let sy = oy as i32 - pad;

            if sx >= 0 && sx < sw as i32 && sy >= 0 && sy < sh as i32 {
                let sp = src.get_pixel(sx as u32, sy as u32);
                if sp[3] > 64 {
                    out.put_pixel(ox, oy, *sp);
                    continue;
                }
            }

            let min_x = (sx - pad).clamp(0, sw as i32 - 1);
            let max_x = (sx + pad).clamp(0, sw as i32 - 1);
            let min_y = (sy - pad).clamp(0, sh as i32 - 1);
            let max_y = (sy + pad).clamp(0, sh as i32 - 1);

            let mut min_dist_sq = f32::MAX;
            for ny in min_y..=max_y {
                for nx in min_x..=max_x {
                    if is_opaque[(ny * sw as i32 + nx) as usize] {
                        let dx = (sx - nx) as f32;
                        let dy = (sy - ny) as f32;
                        let d2 = dx * dx + dy * dy;
                        if d2 < min_dist_sq {
                            min_dist_sq = d2;
                        }
                    }
                }
            }

            if min_dist_sq <= bw2 {
                let dist = min_dist_sq.sqrt();
                match border {
                    StickerBorder::DieCutBranco => {
                        if dist >= bw_f - 1.0 {
                            out.put_pixel(ox, oy, Rgba([120, 120, 120, 255]));
                        } else {
                            out.put_pixel(ox, oy, Rgba([255, 255, 255, 255]));
                        }
                    }
                    StickerBorder::LinhaPreta => {
                        out.put_pixel(ox, oy, Rgba([0, 0, 0, 255]));
                    }
                    StickerBorder::Picote => {
                        if ((ox + oy) / 4) % 2 == 0 {
                            out.put_pixel(ox, oy, Rgba([255, 255, 255, 255]));
                        }
                    }
                    StickerBorder::Serrilhada => {
                        let teeth = (ox % 6) as f32 - 3.0;
                        if dist + teeth.abs() <= bw_f + 1.0 {
                            out.put_pixel(ox, oy, Rgba([255, 255, 255, 255]));
                        }
                    }
                    StickerBorder::Nenhuma => {}
                }
            }
        }
    }

    out
}

fn render_and_blend_sticker(
    canvas: &mut RgbaImage,
    sticker: &StickerElement,
    palette: Option<&Palette>,
    font: &FontRef,
) {
    let mut stk_img = if let Some(ref custom) = sticker.custom_image {
        let (cw, ch) = custom.dimensions();
        if cw > 0 && ch > 0 {
            let scale = (sticker.size as f32 / cw.max(ch) as f32).max(0.01);
            let target_w = ((cw as f32 * scale).round() as u32).max(1);
            let target_h = ((ch as f32 * scale).round() as u32).max(1);
            image::imageops::resize(
                custom.as_ref(),
                target_w,
                target_h,
                image::imageops::FilterType::Nearest,
            )
        } else {
            render_sticker(sticker.kind, sticker.size, font)
        }
    } else {
        render_sticker(sticker.kind, sticker.size, font)
    };
    if sticker.border != StickerBorder::Nenhuma && sticker.border_width > 0 {
        stk_img = apply_sticker_border(&stk_img, sticker.border, sticker.border_width);
    }
    if let Some(pal) = palette {
        if sticker.apply_palette {
            for p in stk_img.pixels_mut() {
                if p[3] > 32 {
                    let closest = pal.nearest_color(RgbColor::new(p[0], p[1], p[2]));
                    *p = Rgba([closest.r, closest.g, closest.b, p[3]]);
                }
            }
        }
    }

    let (cw, ch) = canvas.dimensions();
    let (sw, sh) = stk_img.dimensions();
    let cx_target = if sticker.pos_x <= 1.0 {
        sticker.pos_x * cw as f32
    } else {
        sticker.pos_x
    };
    let cy_target = if sticker.pos_y <= 1.0 {
        sticker.pos_y * ch as f32
    } else {
        sticker.pos_y
    };

    let rad = (sticker.rotation as f32).to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let half_w = sw as f32 / 2.0;
    let half_h = sh as f32 / 2.0;
    let radius = (half_w * half_w + half_h * half_h).sqrt();

    let x0 = (cx_target - radius).max(0.0) as i32;
    let x1 = (cx_target + radius).min(cw as f32 - 1.0) as i32;
    let y0 = (cy_target - radius).max(0.0) as i32;
    let y1 = (cy_target + radius).min(ch as f32 - 1.0) as i32;

    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f32 - cx_target;
            let dy = py as f32 - cy_target;
            let sx_f = half_w + (dx * cos_a + dy * sin_a);
            let sy_f = half_h + (-dx * sin_a + dy * cos_a);

            let sx = sx_f.round() as i32;
            let sy = sy_f.round() as i32;

            if sx >= 0 && sx < sw as i32 && sy >= 0 && sy < sh as i32 {
                let sp = stk_img.get_pixel(sx as u32, sy as u32);
                if sp[3] > 0 {
                    if sp[3] == 255 {
                        canvas.put_pixel(px as u32, py as u32, *sp);
                    } else {
                        let cp = canvas.get_pixel(px as u32, py as u32);
                        let alpha = sp[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        let r = (sp[0] as f32 * alpha + cp[0] as f32 * inv_alpha).round() as u8;
                        let g = (sp[1] as f32 * alpha + cp[1] as f32 * inv_alpha).round() as u8;
                        let b = (sp[2] as f32 * alpha + cp[2] as f32 * inv_alpha).round() as u8;
                        canvas.put_pixel(px as u32, py as u32, Rgba([r, g, b, 255]));
                    }
                }
            }
        }
    }
}
