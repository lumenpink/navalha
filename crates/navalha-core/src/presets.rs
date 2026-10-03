use crate::dither::DitherAlgorithm;
use crate::guillotine::GuillotineConfig;
use crate::palette::{Palette, RgbColor};
use crate::prep::PrepConfig;
use crate::press::{BannerFontFamily, DamageEffect, DamageElement, PosterComposition, PunkOverlayConfig, TarjaBorderFinish, TarjaEffect, TextElement};
use crate::scale::ScaleFactor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AestheticPreset {
    PunkRiot,
    NavalhaSata,
    XeroxPodre,
    ViagemSolitaria,
    LapaNoturna,
    TeatroOperario,
    Risografia,
    FanzineDigital,
    AsfaltoAlerta,
    Vga256,
    Cga,
    OldMacPopArt,
    AsciiArt,
    TabloidePopular,
    Brutalismo,
    TrashPolka,
    AtariGaming,
    Mimeografo,
    Xilogravura,
    GlitchCrt,
}

impl AestheticPreset {
    pub fn all() -> &'static [AestheticPreset] {
        &[
            Self::PunkRiot,
            Self::NavalhaSata,
            Self::XeroxPodre,
            Self::ViagemSolitaria,
            Self::LapaNoturna,
            Self::TeatroOperario,
            Self::Risografia,
            Self::FanzineDigital,
            Self::AsfaltoAlerta,
            Self::Vga256,
            Self::Cga,
            Self::OldMacPopArt,
            Self::AsciiArt,
            Self::TabloidePopular,
            Self::Brutalismo,
            Self::TrashPolka,
            Self::AtariGaming,
            Self::Mimeografo,
            Self::Xilogravura,
            Self::GlitchCrt,
        ]
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::PunkRiot => "punk-riot",
            Self::NavalhaSata => "navalha-sata",
            Self::XeroxPodre => "xerox-podre",
            Self::ViagemSolitaria => "viagem-solitaria",
            Self::LapaNoturna => "lapa-noturna",
            Self::TeatroOperario => "teatro-operario",
            Self::Risografia => "risografia",
            Self::FanzineDigital => "fanzine-digital",
            Self::AsfaltoAlerta => "asfalto-alerta",
            Self::Vga256 => "vga256",
            Self::Cga => "cga",
            Self::OldMacPopArt => "old-mac-pop-art",
            Self::AsciiArt => "ascii-art",
            Self::TabloidePopular => "tabloide-popular",
            Self::Brutalismo => "brutalismo",
            Self::TrashPolka => "trash-polka",
            Self::AtariGaming => "atari-gaming",
            Self::Mimeografo => "mimeografo",
            Self::Xilogravura => "xilogravura",
            Self::GlitchCrt => "glitch-crt",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PunkRiot => "Punk Riot / Fanzine Anarquista (Claudia Wonder)",
            Self::NavalhaSata => "Navalha Satã / Linóleo & Ouro Queimado (Madame Satã)",
            Self::XeroxPodre => "Xerox Podre / Fotocopiadora Velha (Indianarae Siqueira)",
            Self::ViagemSolitaria => "Viagem Solitária / Kraft & Olivetti (João W. Nery)",
            Self::LapaNoturna => "Lapa Noturna / Duotone Ácido de Rua (Linn da Quebrada)",
            Self::TeatroOperario => "Teatro Operário / Cartaz Construtivista (Leo Moreira Sá)",
            Self::Risografia => "Risografia Crua / Ponto de Impressão (Erika Hilton)",
            Self::FanzineDigital => "Fanzine Digital / Cyber-BBS Anos 90 (Amara Moira)",
            Self::AsfaltoAlerta => "Asfalto & Ouro / Alerta Industrial (Kátia Tapety)",
            Self::Vga256 => "VGA 256 / Modo 13h (Márcia Pantera)",
            Self::Cga => "CGA / IBM PC 1981 (Neon Cunha)",
            Self::OldMacPopArt => "Old Mac Pop Art / Warhol & Atkinson (Rogéria)",
            Self::AsciiArt => "ASCII Art / Cordeiro 1971 CRT (Laerte Coutinho)",
            Self::TabloidePopular => "Tabloide Popular / Manchete de Rotativa (Luana Muniz)",
            Self::Brutalismo => "Brutalismo / Concreto & Tipografia Maciça (Drica D'arc Meirelles)",
            Self::TrashPolka => "Trash Polka / Realismo & Sangue (Renata Carvalho)",
            Self::AtariGaming => "Atari Gaming / Stella TIA & Scanlines CRT (Jovanna Baby)",
            Self::Mimeografo => "Mimeógrafo / Violeta Anilina & Álcool (Keila Simpson)",
            Self::Xilogravura => "Xilogravura / Talhe em Madeira & Cordel (Xica Manicongo)",
            Self::GlitchCrt => "Glitch CRT / Sinal Pirata & Aberração Cromática (Robeyoncé Lima)",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let norm = s.to_lowercase().replace(['-', '_', ' '], "");
        match norm.as_str() {
            "punkriot" | "punk" | "riot" | "navalha" | "claudiawonder" => Some(Self::PunkRiot),
            "navalhasata" | "sata" | "madamesata" | "linoleo" => Some(Self::NavalhaSata),
            "xeroxpodre" | "xerox" | "fotocopiadora" | "indianaraesiqueira" | "indianarae" => Some(Self::XeroxPodre),
            "viagemsolitaria" | "nery" | "joaownery" | "kraft" | "olivetti" => Some(Self::ViagemSolitaria),
            "lapanoturna" | "lapa" | "cyber" | "neon" | "linndaquebrada" | "linn" => Some(Self::LapaNoturna),
            "teatrooperario" | "leosa" | "leomoreirasa" | "teatro" | "operario" | "construtivista" => Some(Self::TeatroOperario),
            "risografia" | "riso" | "risografiacrua" | "erikahilton" => Some(Self::Risografia),
            "fanzinedigital" | "amaramoir" | "amara" | "amramoira" | "fanzine" | "cyberbbs" | "bbs" => Some(Self::FanzineDigital),
            "asfaltoalerta" | "asfalto" | "alerta" | "ouro" | "asfaltoeouro" | "katiatapety" => Some(Self::AsfaltoAlerta),
            "vga256" | "vga" | "mode13h" | "marciapantera" => Some(Self::Vga256),
            "cga" | "cgapalette1" | "ibmpc" | "neoncunha" => Some(Self::Cga),
            "oldmacpopart" | "oldmac" | "popart" | "warhol" | "atkinson" | "rogeria" => Some(Self::OldMacPopArt),
            "asciiart" | "ascii" | "cordeiro" | "terminal" | "laerte" | "laertecoutinho" => Some(Self::AsciiArt),
            "tabloidepopular" | "tabloide" | "noticiaspopulares" | "imprensa" | "luanamuniz" => Some(Self::TabloidePopular),
            "brutalismo" | "brutalista" | "dricadarc" | "dricadarcmeirelles" => Some(Self::Brutalismo),
            "trashpolka" | "trash" | "polka" | "renatacarvalho" => Some(Self::TrashPolka),
            "atarigaming" | "atari" | "tia" | "stella" | "jovannababy" => Some(Self::AtariGaming),
            "mimeografo" | "mimeo" | "anilina" | "keilasimpson" | "keila" => Some(Self::Mimeografo),
            "xilogravura" | "xilo" | "cordel" | "xicamanicongo" => Some(Self::Xilogravura),
            "glitchcrt" | "glitch" | "crt" | "robeyonce" | "robeyoncelima" => Some(Self::GlitchCrt),
            _ => None,
        }
    }

    pub fn default_banner_element(&self, phrase: &str) -> TextElement {
        let phrase_upper = phrase.trim().to_uppercase();
        match self {
            Self::PunkRiot => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(255, 255, 255),
                bg_tarja: Some(RgbColor::new(0, 0, 0)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: -2,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 2,
                border_color: Some(RgbColor::new(255, 0, 51)),
                word_jitter_rotation: 6,
                word_tarja_cut: true,
                ..Default::default()
            },
            Self::NavalhaSata => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(255, 255, 255),
                bg_tarja: Some(RgbColor::new(26, 20, 18)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Serif,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 2,
                border_color: Some(RgbColor::new(212, 136, 58)),
                ..Default::default()
            },
            Self::XeroxPodre => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(230, 226, 216),
                bg_tarja: Some(RgbColor::new(20, 18, 16)),
                tarja_padding: 14,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Xerox,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Picote,
                border_width: 3,
                border_color: Some(RgbColor::new(50, 45, 40)),
                ..Default::default()
            },
            Self::ViagemSolitaria => TextElement {
                text: phrase_upper,
                font_size: 54.0,
                stretch_x: 1.0,
                stretch_y: 1.6,
                color: RgbColor::new(245, 235, 225),
                bg_tarja: Some(RgbColor::new(43, 29, 12)),
                tarja_padding: 14,
                pos_x: 0.5,
                pos_y: 0.86,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(200, 168, 112)),
                ..Default::default()
            },
            Self::LapaNoturna => TextElement {
                text: phrase_upper,
                font_size: 54.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(0, 255, 255),
                bg_tarja: Some(RgbColor::new(10, 10, 14)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 2,
                border_color: Some(RgbColor::new(255, 0, 128)),
                wave_amplitude: 5.0,
                wave_wavelength: 200.0,
                ..Default::default()
            },
            Self::TeatroOperario => TextElement {
                text: phrase_upper,
                font_size: 54.0,
                stretch_x: 1.1,
                stretch_y: 1.5,
                color: RgbColor::new(230, 226, 216),
                bg_tarja: Some(RgbColor::new(17, 17, 17)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 4,
                border_color: Some(RgbColor::new(179, 27, 27)),
                ..Default::default()
            },
            Self::Risografia => TextElement {
                text: phrase_upper,
                font_size: 54.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(255, 242, 0),
                bg_tarja: Some(RgbColor::new(255, 72, 176)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.86,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Sans,
                border_finish: TarjaBorderFinish::Picote,
                border_width: 2,
                border_color: Some(RgbColor::new(0, 85, 255)),
                ..Default::default()
            },
            Self::FanzineDigital => TextElement {
                text: phrase_upper,
                font_size: 50.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(0, 17, 136),
                bg_tarja: Some(RgbColor::new(255, 238, 0)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(0, 17, 136)),
                wave_amplitude: 6.0,
                wave_wavelength: 160.0,
                ..Default::default()
            },
            Self::AsfaltoAlerta => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.1,
                stretch_y: 1.5,
                color: RgbColor::new(17, 17, 17),
                bg_tarja: Some(RgbColor::new(255, 204, 0)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 2,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 4,
                border_color: Some(RgbColor::new(17, 17, 17)),
                word_jitter_rotation: 4,
                word_tarja_cut: true,
                ..Default::default()
            },
            Self::Vga256 => TextElement {
                text: phrase_upper,
                font_size: 50.0,
                stretch_x: 1.0,
                stretch_y: 1.6,
                color: RgbColor::new(255, 255, 85),
                bg_tarja: Some(RgbColor::new(0, 0, 0)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(0, 0, 170)),
                ..Default::default()
            },
            Self::Cga => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.0,
                stretch_y: 1.6,
                color: RgbColor::new(85, 255, 255),
                bg_tarja: Some(RgbColor::new(0, 0, 0)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 2,
                border_color: Some(RgbColor::new(255, 85, 255)),
                ..Default::default()
            },
            Self::OldMacPopArt => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.1,
                stretch_y: 1.5,
                color: RgbColor::new(0, 0, 0),
                bg_tarja: Some(RgbColor::new(255, 255, 255)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Sans,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 3,
                border_color: Some(RgbColor::new(255, 105, 180)),
                ..Default::default()
            },
            Self::Brutalismo => TextElement {
                text: phrase_upper,
                font_size: 54.0,
                stretch_x: 0.9,
                stretch_y: 1.8,
                color: RgbColor::new(250, 250, 250), // Branco Gesso
                bg_tarja: Some(RgbColor::new(10, 10, 10)), // Preto Piche maciço
                tarja_padding: 16,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 4,
                border_color: Some(RgbColor::new(10, 10, 10)),
                ..Default::default()
            },
            Self::TrashPolka => TextElement {
                text: phrase_upper,
                font_size: 48.0,
                stretch_x: 1.0,
                stretch_y: 1.4,
                color: RgbColor::new(255, 255, 255), // Branco cirúrgico
                bg_tarja: Some(RgbColor::new(186, 12, 47)), // Carmim Trash Polka
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.86,
                center_x: true,
                rotation: 3,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(17, 17, 17)),
                word_jitter_rotation: 8,
                word_tarja_cut: true,
                ..Default::default()
            },
            Self::AtariGaming => TextElement {
                text: phrase_upper,
                font_size: 46.0,
                stretch_x: 1.1,
                stretch_y: 1.5,
                color: RgbColor::new(252, 252, 112), // Amarelo Stella TIA
                bg_tarja: Some(RgbColor::new(0, 0, 0)), // Fundo preto puro
                tarja_padding: 10,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 3,
                border_color: Some(RgbColor::new(208, 108, 108)),
                ..Default::default()
            },
            Self::Mimeografo => TextElement {
                text: phrase_upper,
                font_size: 48.0,
                stretch_x: 1.0,
                stretch_y: 1.5,
                color: RgbColor::new(244, 239, 230), // Papel manilha
                bg_tarja: Some(RgbColor::new(67, 44, 122)), // Violeta anilina
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Xerox,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Picote,
                border_width: 2,
                border_color: Some(RgbColor::new(42, 25, 84)),
                ..Default::default()
            },
            Self::Xilogravura => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.1,
                stretch_y: 1.6,
                color: RgbColor::new(214, 196, 165), // Kraft
                bg_tarja: Some(RgbColor::new(19, 17, 16)), // Preto de fumo
                tarja_padding: 14,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 4,
                border_color: Some(RgbColor::new(214, 196, 165)),
                ..Default::default()
            },
            Self::GlitchCrt => TextElement {
                text: phrase_upper,
                font_size: 50.0,
                stretch_x: 1.2,
                stretch_y: 1.4,
                color: RgbColor::new(0, 255, 204), // Fósforo verde/ciano
                bg_tarja: Some(RgbColor::new(13, 15, 18)), // Fundo ruído escuro
                tarja_padding: 10,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(255, 0, 51)),
                wave_amplitude: 8.0,
                wave_wavelength: 180.0,
                ..Default::default()
            },
            Self::AsciiArt => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.0,
                stretch_y: 1.6,
                color: RgbColor::new(0, 255, 102),
                bg_tarja: Some(RgbColor::new(5, 10, 5)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: 0,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Monospace,
                border_finish: TarjaBorderFinish::Tracejada,
                border_width: 2,
                border_color: Some(RgbColor::new(0, 255, 102)),
                ..Default::default()
            },
            Self::TabloidePopular => TextElement {
                text: phrase_upper,
                font_size: 52.0,
                stretch_x: 1.1,
                stretch_y: 1.5,
                color: RgbColor::new(255, 255, 255),
                bg_tarja: Some(RgbColor::new(10, 10, 10)),
                tarja_padding: 12,
                pos_x: 0.5,
                pos_y: 0.88,
                center_x: true,
                rotation: -1,
                tarja_effect: TarjaEffect::Solida,
                font_family: BannerFontFamily::Impact,
                border_finish: TarjaBorderFinish::Solida,
                border_width: 3,
                border_color: Some(RgbColor::new(255, 204, 0)),
                word_jitter_rotation: 5,
                word_tarja_cut: true,
                ..Default::default()
            },
        }
    }

    pub fn build_pipeline_and_composition_dual(
        &self,
        phrase_bottom: &str,
        phrase_top: Option<&str>,
    ) -> (
        GuillotineConfig,
        PrepConfig,
        DitherAlgorithm,
        Palette,
        ScaleFactor,
        PosterComposition,
    ) {
        let (g, pr, dt, pal, sc, mut comp) = self.build_pipeline_and_composition(phrase_bottom);

        // Se uma frase superior foi especificada, herda a estilização da tarja do preset
        if let Some(top) = phrase_top {
            let top_clean = top.trim().to_uppercase();
            if !top_clean.is_empty() {
                if let Some(base_tarja) = comp.foreground_texts.first().cloned() {
                    let mut top_elem = base_tarja;
                    top_elem.text = top_clean;
                    top_elem.pos_x = 0.5;
                    top_elem.pos_y = 0.08;
                    top_elem.center_x = true;
                    comp.foreground_texts.insert(0, top_elem);
                }
            }
        }

        // Se a frase inferior estiver vazia, remove o elemento residual vazio
        if phrase_bottom.trim().is_empty() {
            comp.foreground_texts.retain(|t| !t.text.trim().is_empty());
        }

        (g, pr, dt, pal, sc, comp)
    }

    pub fn build_pipeline_and_composition(
        &self,
        phrase: &str,
    ) -> (
        GuillotineConfig,
        PrepConfig,
        DitherAlgorithm,
        Palette,
        ScaleFactor,
        PosterComposition,
    ) {
        let phrase_upper = phrase.trim().to_uppercase();

        match self {
            Self::PunkRiot => {
                let palette = Palette::classic_punk();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.88,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 2.5,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.15,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Atkinson;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(255, 0, 51), // Fundo vermelho punk
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 96.0,
                        stretch_x: 1.2,
                        stretch_y: 2.0,
                        color: RgbColor::new(0, 0, 0), // Letras gigantes pretas sangrando no fundo
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.15,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 0, 0),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::XeroxPodre => {
                let palette = Palette::xerox_podre();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.82,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(640),
                    unsharp_strength: 3.2,
                    unsharp_radius: 3,
                    levels_shadow_cutoff: 0.25,
                    levels_highlight_cutoff: 0.75,
                    levels_gamma: 0.8,
                };
                let dither = DitherAlgorithm::Bayer8x8;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(245, 242, 235), // Papel oxidado
                    background_text: None,
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(28, 25, 23),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::LapaNoturna => {
                let palette = Palette::lapa_noturna();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(800),
                    unsharp_strength: 2.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.14,
                    levels_highlight_cutoff: 0.90,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::FloydSteinberg;
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(11, 11, 14), // Preto asfalto
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 88.0,
                        stretch_x: 1.1,
                        stretch_y: 2.2,
                        color: RgbColor::new(255, 0, 127), // Rosa choque
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.12,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 240, 255),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Risografia => {
                let palette = Palette::risografia_crua();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(760),
                    unsharp_strength: 2.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.15,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Halftone { angle_deg: 45, cell_size: 6 };
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(255, 242, 0), // Amarelo riso
                    background_text: None,
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 56, 168),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::AsfaltoAlerta => {
                let palette = Palette::asfalto_ouro();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 2.8,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.20,
                    levels_highlight_cutoff: 0.80,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Atkinson;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(17, 17, 17), // Preto piche
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 90.0,
                        stretch_x: 1.2,
                        stretch_y: 1.8,
                        color: RgbColor::new(35, 35, 35), // Marca d'água pesada
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.15,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(255, 204, 0),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Vga256 => {
                let palette = Palette::vga_256();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                // Low-res clássica do Modo 13h (320px base)
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(320),
                    unsharp_strength: 3.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.12,
                    levels_highlight_cutoff: 0.88,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Bayer4x4;
                let scale = ScaleFactor::ThreeX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(0, 0, 170), // Azul clássico DOS (#0000AA)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 92.0,
                        stretch_x: 1.2,
                        stretch_y: 2.0,
                        color: RgbColor::new(0, 0, 85), // Azul escuro sangrado no fundo
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(85, 255, 255), // Ciano brilhante VGA (#55FFFF)
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Cga => {
                let palette = Palette::cga_palette1();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                // Resolução baixa CGA 320x200
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(320),
                    unsharp_strength: 3.2,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.10,
                    levels_highlight_cutoff: 0.90,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Bayer4x4;
                let scale = ScaleFactor::ThreeX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(0, 0, 0), // Preto sólido CGA
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 94.0,
                        stretch_x: 1.2,
                        stretch_y: 2.0,
                        color: RgbColor::new(255, 85, 255), // Magenta CGA (#FF55FF)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(255, 255, 255), // Branco CGA (#FFFFFF)
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::OldMacPopArt => {
                let palette = Palette::pop_art_mac();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.86,
                    invert: true,
                    background_fill: None,
                };
                // Resolução Macintosh 128k (512px)
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(512),
                    unsharp_strength: 2.6,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.15,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Atkinson;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(255, 230, 0), // Amarelo Serigráfico Pop (#FFE600)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 96.0,
                        stretch_x: 1.2,
                        stretch_y: 1.8,
                        color: RgbColor::new(255, 0, 127), // Rosa Pop Art Fluorescente (#FF007F)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 210, 255), // Azul Turquesa (#00D2FF)
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Brutalismo => {
                let palette = Palette::brutalismo();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 3.5, // 3.5x unsharp agressivo
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.15,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Threshold; // Binarização estrita por média ponderada
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(229, 229, 229), // #E5E5E5 Concreto aparente
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 98.0,
                        stretch_x: 0.9,
                        stretch_y: 2.2,
                        color: RgbColor::new(10, 10, 10), // Preto Piche maciço
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.12,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        font_family: BannerFontFamily::Impact,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(10, 10, 10),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::TrashPolka => {
                let palette = Palette::trash_polka();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.88,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 2.5,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.12,
                    levels_highlight_cutoff: 0.88,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::FloydSteinberg; // 4 níveis de cinza com paleta trash_polka
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(255, 255, 255), // Branco cirúrgico puro
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 94.0,
                        stretch_x: 1.1,
                        stretch_y: 1.9,
                        color: RgbColor::new(186, 12, 47), // Vermelho Carmim (#BA0C2F)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![
                        self.default_banner_element(&phrase_upper),
                        TextElement {
                            text: "LAT: -23.5505 / LONG: -46.6333 // ELEV: 760M".to_string(),
                            font_size: 16.0,
                            stretch_x: 1.0,
                            stretch_y: 1.0,
                            color: RgbColor::new(17, 17, 17),
                            bg_tarja: None,
                            tarja_padding: 0,
                            pos_x: 0.5,
                            pos_y: 0.94,
                            center_x: true,
                            rotation: 0,
                            tarja_effect: TarjaEffect::Solida,
                            font_family: BannerFontFamily::Monospace,
                            ..Default::default()
                        },
                    ],
                    overlay: PunkOverlayConfig {
                        damage_elements: vec![
                            DamageElement::new(DamageEffect::Sangue, 0.20, 0.35, 1.2),
                            DamageElement::new(DamageEffect::Tiro, 0.82, 0.28, 1.0),
                            DamageElement::new(DamageEffect::Sangue, 0.75, 0.70, 0.9),
                        ],
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(186, 12, 47), // Vetores Carmim
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::AtariGaming => {
                let palette = Palette::atari_gaming();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(320),
                    unsharp_strength: 1.5,
                    unsharp_radius: 1,
                    levels_shadow_cutoff: 0.05,
                    levels_highlight_cutoff: 0.95,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::AtariGaming;
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(0, 0, 0), // Fundo preto puro CRT
                    background_text: Some(TextElement {
                        text: "ATARI 2600 TIA".to_string(),
                        font_size: 72.0,
                        stretch_x: 1.2,
                        stretch_y: 1.6,
                        color: RgbColor::new(68, 68, 0), // Ouro CRT
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.10,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        font_family: BannerFontFamily::Monospace,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: false,
                        vector_color: RgbColor::new(208, 108, 108),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Mimeografo => {
                let palette = Palette::mimeografo();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(640),
                    unsharp_strength: 2.2,
                    unsharp_radius: 1,
                    levels_shadow_cutoff: 0.10,
                    levels_highlight_cutoff: 0.90,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Mimeografo;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(244, 239, 230), // Papel manilha / palha (#F4EFE6)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 92.0,
                        stretch_x: 1.0,
                        stretch_y: 2.0,
                        color: RgbColor::new(67, 44, 122), // Tinta Violeta Metil (#432C7A)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: vec![
                            DamageElement::new(DamageEffect::Tinta, 0.50, 0.05, 1.0),
                            DamageElement::new(DamageEffect::Chuva, 0.50, 0.50, 0.8),
                        ],
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(42, 25, 84), // Saturação de álcool
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::Xilogravura => {
                let palette = Palette::xilogravura();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.84,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 3.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.12,
                    levels_highlight_cutoff: 0.88,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::SauvolaWoodcut;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(214, 196, 165), // Papel kraft queimado (#D6C4A5)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 96.0,
                        stretch_x: 1.1,
                        stretch_y: 2.1,
                        color: RgbColor::new(19, 17, 16), // Preto de fumo (#131110)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: vec![
                            DamageElement::new(DamageEffect::Faca, 0.50, 0.35, 1.2),
                        ],
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(19, 17, 16),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::GlitchCrt => {
                let palette = Palette::glitch_crt();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(640),
                    unsharp_strength: 2.0,
                    unsharp_radius: 1,
                    levels_shadow_cutoff: 0.08,
                    levels_highlight_cutoff: 0.92,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::GlitchCrt;
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(13, 15, 18), // Fundo ruído escuro (#0D0F12)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 92.0,
                        stretch_x: 1.2,
                        stretch_y: 1.8,
                        color: RgbColor::new(0, 255, 204), // Fósforo P31 Verde/Ciano (#00FFCC)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 255, 204),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::AsciiArt => {
                let palette = Palette::terminal_green();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(82),
                    unsharp_strength: 2.5,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.08,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::AsciiArt;
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(5, 10, 5), // Fundo preto CRT
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 94.0,
                        stretch_x: 1.2,
                        stretch_y: 2.0,
                        color: RgbColor::new(0, 68, 20), // Marca d'água verde escuro fósforo
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(0, 255, 102),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::TabloidePopular => {
                let palette = Palette::tabloide_popular();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.84,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 3.2, // Unsharp agressivo de impressão em rotativa
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.16,
                    levels_highlight_cutoff: 0.84,
                    levels_gamma: 0.95,
                };
                let dither = DitherAlgorithm::FloydSteinberg;
                let scale = ScaleFactor::OneX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(232, 230, 223), // Papel jornal cinza claro (#E8E6DF)
                    background_text: Some(TextElement {
                        text: "URGENTE".to_string(),
                        font_size: 96.0,
                        stretch_x: 1.1,
                        stretch_y: 1.6,
                        color: RgbColor::new(255, 230, 0), // Amarelo sensacionalista de fundo
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.12,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![
                        TextElement {
                            text: "EDIÇÃO EXTRA // EXCLUSIVO".to_string(),
                            font_size: 32.0,
                            stretch_x: 1.0,
                            stretch_y: 1.3,
                            color: RgbColor::new(255, 255, 255),
                            bg_tarja: Some(RgbColor::new(230, 0, 0)), // Tarja vermelha manchete de rotativa
                            tarja_padding: 8,
                            pos_x: 0.5,
                            pos_y: 0.05,
                            center_x: true,
                            rotation: 0,
                            tarja_effect: TarjaEffect::Solida,
                            font_family: BannerFontFamily::Impact,
                            border_finish: TarjaBorderFinish::Solida,
                            border_width: 2,
                            border_color: Some(RgbColor::new(255, 204, 0)),
                            ..Default::default()
                        },
                        self.default_banner_element(&phrase_upper),
                    ],
                    overlay: PunkOverlayConfig { damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true, // Marcas de registro industrial de gráfica
                        vector_color: RgbColor::new(10, 10, 10),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::NavalhaSata => {
                let palette = Palette::linoleo_ouro();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.86,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 3.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.12,
                    levels_highlight_cutoff: 0.88,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Atkinson;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(212, 136, 58), // Ouro cru queimado (#D4883A)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 96.0,
                        stretch_x: 1.2,
                        stretch_y: 2.0,
                        color: RgbColor::new(26, 20, 18), // Carvão maciço de xilogravura
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.15,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(26, 20, 18),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::ViagemSolitaria => {
                let palette = Palette::kraft_olivetti();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.82,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(640),
                    unsharp_strength: 2.8,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.18,
                    levels_highlight_cutoff: 0.82,
                    levels_gamma: 0.85,
                };
                let dither = DitherAlgorithm::FloydSteinberg;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(200, 168, 112), // Papel kraft cru (#C8A870)
                    background_text: None,
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: true,
                        registration_marks: true,
                        vector_color: RgbColor::new(43, 29, 12),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::TeatroOperario => {
                let palette = Palette::teatro_operario();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: true,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(720),
                    unsharp_strength: 3.0,
                    unsharp_radius: 2,
                    levels_shadow_cutoff: 0.15,
                    levels_highlight_cutoff: 0.85,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Bayer4x4;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(230, 226, 216), // Papel jornal cru (#E6E2D8)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 98.0,
                        stretch_x: 1.2,
                        stretch_y: 2.1,
                        color: RgbColor::new(179, 27, 27), // Vermelho carmesim construtivista
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(17, 17, 17),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }

            Self::FanzineDigital => {
                let palette = Palette::cyber_bbs();
                let guillotine = GuillotineConfig {
                    auto_threshold: false,
                    enabled: false,
                    threshold: 0.85,
                    invert: true,
                    background_fill: None,
                };
                let prep = PrepConfig {
                    rotation: 0,
                    zoom: 1.0,
                    target_width: Some(640),
                    unsharp_strength: 2.4,
                    unsharp_radius: 1,
                    levels_shadow_cutoff: 0.10,
                    levels_highlight_cutoff: 0.90,
                    levels_gamma: 1.0,
                };
                let dither = DitherAlgorithm::Bayer8x8;
                let scale = ScaleFactor::TwoX;

                let comp = PosterComposition {
                    stickers: Vec::new(),
                    subject_offset_x: 0,
                    subject_offset_y: 0,
                    palette: Some(palette.clone()),
                    canvas_width: 800,
                    canvas_height: 1100,
                    background_color: RgbColor::new(0, 17, 136), // Azul ultramarino BBS (#001188)
                    background_text: Some(TextElement {
                        text: phrase_upper.clone(),
                        font_size: 92.0,
                        stretch_x: 1.1,
                        stretch_y: 2.0,
                        color: RgbColor::new(255, 238, 0), // Amarelo CRT (#FFEE00)
                        bg_tarja: None,
                        tarja_padding: 0,
                        pos_x: 0.5,
                        pos_y: 0.14,
                        center_x: true,
                        rotation: 0,
                        tarja_effect: TarjaEffect::Solida,
                        ..Default::default()
                    }),
                    foreground_texts: vec![self.default_banner_element(&phrase_upper)],
                    overlay: PunkOverlayConfig {
                        damage_elements: Vec::new(),
                        damages: Vec::new(),
                        crop_frame: true,
                        blade_scratches: false,
                        registration_marks: true,
                        vector_color: RgbColor::new(255, 255, 255),
                    },
                };

                (guillotine, prep, dither, palette, scale, comp)
            }
        }
    }
}
