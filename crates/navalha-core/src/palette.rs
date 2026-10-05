use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() == 3 {
            let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).ok()?;
            Some(Self { r, g, b })
        } else if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
            Some(Self { r, g, b })
        } else {
            None
        }
    }

    pub fn parse_color_str(s: &str) -> Option<Self> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "white" | "branco" => Some(Self::new(255, 255, 255)),
            "black" | "preto" => Some(Self::new(0, 0, 0)),
            "red" | "vermelho" => Some(Self::new(255, 0, 51)),
            "yellow" | "amarelo" => Some(Self::new(255, 238, 0)),
            "cyan" | "ciano" => Some(Self::new(0, 255, 255)),
            "green" | "verde" => Some(Self::new(0, 255, 102)),
            "blue" | "azul" => Some(Self::new(0, 68, 255)),
            "gold" | "ouro" => Some(Self::new(212, 136, 58)),
            _ => Self::from_hex(s),
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn luminance(&self) -> f32 {
        // Fórmula de luminância perceptual ITU-R BT.601
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    /// Distância euclidiana ponderada no espaço de cor RGB
    pub fn distance_squared(&self, other: &Self) -> u32 {
        let dr = self.r as i32 - other.r as i32;
        let dg = self.g as i32 - other.g as i32;
        let db = self.b as i32 - other.b as i32;
        // Ponderação perceptual humana: olho é mais sensível ao verde, depois vermelho e azul
        (dr * dr * 3 + dg * dg * 4 + db * db * 2) as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<RgbColor>,
}

impl Palette {
    pub fn new(name: impl Into<String>, colors: Vec<RgbColor>) -> Self {
        Self {
            name: name.into(),
            colors,
        }
    }

    pub fn nearest_color(&self, c: RgbColor) -> RgbColor {
        if self.colors.is_empty() {
            return c;
        }
        let mut best_color = self.colors[0];
        let mut min_dist = u32::MAX;

        for &palette_col in &self.colors {
            let d = c.distance_squared(&palette_col);
            if d < min_dist {
                min_dist = d;
                best_color = palette_col;
            }
        }
        best_color
    }

    pub fn parse_csv(csv: &str) -> Option<Self> {
        let mut colors = Vec::new();
        for part in csv.split(',') {
            let p = part.trim();
            if !p.is_empty() {
                if let Some(c) = RgbColor::from_hex(p) {
                    colors.push(c);
                }
            }
        }
        if colors.is_empty() {
            None
        } else {
            Some(Self::new("Personalizada", colors))
        }
    }

    pub fn classic_punk() -> Self {
        Self::new(
            "Navalha Clássica",
            vec![
                RgbColor::new(0, 0, 0),         // #000000 Preto
                RgbColor::new(255, 255, 255),   // #FFFFFF Branco
                RgbColor::new(255, 0, 51),       // #FF0033 Vermelho Punk
            ],
        )
    }

    pub fn lapa_noturna() -> Self {
        Self::new(
            "Lapa Noturna",
            vec![
                RgbColor::new(11, 11, 14),      // #0B0B0E Preto asfalto
                RgbColor::new(255, 0, 127),     // #FF007F Magenta neon
                RgbColor::new(0, 240, 255),     // #00F0FF Ciano elétrico
            ],
        )
    }

    pub fn xerox_podre() -> Self {
        Self::new(
            "Xerox Podre",
            vec![
                RgbColor::new(28, 25, 23),      // #1C1917 Toner oxidado
                RgbColor::new(245, 242, 235),   // #F5F2EB Papel jornal velho
            ],
        )
    }

    pub fn risografia_crua() -> Self {
        Self::new(
            "Risografia Crua",
            vec![
                RgbColor::new(0, 56, 168),      // #0038A8 Azul Royal
                RgbColor::new(255, 72, 176),    // #FF48B0 Rosa Fluorescente
                RgbColor::new(255, 242, 0),     // #FFF200 Amarelo Médio
            ],
        )
    }

    pub fn asfalto_ouro() -> Self {
        Self::new(
            "Asfalto & Ouro",
            vec![
                RgbColor::new(17, 17, 17),      // #111111 Preto piche
                RgbColor::new(255, 204, 0),     // #FFCC00 Amarelo advertência
            ],
        )
    }

    pub fn vga_256() -> Self {
        let mut colors = Vec::with_capacity(256);

        // 1. 16 cores básicas padrão do BIOS / CGA / EGA / VGA
        let ega16 = [
            (0, 0, 0),       // 0: Preto
            (0, 0, 170),     // 1: Azul
            (0, 170, 0),     // 2: Verde
            (0, 170, 170),   // 3: Ciano
            (170, 0, 0),     // 4: Vermelho
            (170, 0, 170),   // 5: Magenta
            (170, 85, 0),    // 6: Marrom
            (170, 170, 170), // 7: Cinza Claro
            (85, 85, 85),    // 8: Cinza Escuro
            (85, 85, 255),   // 9: Azul Brilhante
            (85, 255, 85),   // 10: Verde Brilhante
            (85, 255, 255),  // 11: Ciano Brilhante
            (255, 85, 85),   // 12: Vermelho Brilhante
            (255, 85, 255),  // 13: Magenta Brilhante
            (255, 255, 85),  // 14: Amarelo
            (255, 255, 255), // 15: Branco Brilhante
        ];
        for (r, g, b) in ega16 {
            colors.push(RgbColor::new(r, g, b));
        }

        // 2. Cubo de cores 6x6x6 (216 cores)
        let levels = [0, 51, 102, 153, 204, 255];
        for &r in &levels {
            for &g in &levels {
                for &b in &levels {
                    colors.push(RgbColor::new(r, g, b));
                }
            }
        }

        // 3. Rampa de 24 tons de cinza monocromáticos (16 + 216 + 24 = 256)
        for i in 0..24 {
            let gray = (8 + i * 10).min(248) as u8;
            colors.push(RgbColor::new(gray, gray, gray));
        }

        Self::new("VGA 256", colors)
    }

    pub fn cga_palette1() -> Self {
        Self::new(
            "CGA Paleta 1 (Alto Brilho - Neon Cunha)",
            vec![
                RgbColor::new(0, 0, 0),        // #000000 Preto
                RgbColor::new(85, 255, 255),   // #55FFFF Ciano Brilhante
                RgbColor::new(255, 85, 255),   // #FF55FF Magenta Brilhante
                RgbColor::new(255, 255, 255),  // #FFFFFF Branco
            ],
        )
    }

    pub fn cga_palette0() -> Self {
        Self::new(
            "CGA Paleta 0 (Alto Brilho)",
            vec![
                RgbColor::new(0, 0, 0),        // #000000 Preto
                RgbColor::new(85, 255, 85),    // #55FF55 Verde Brilhante
                RgbColor::new(255, 85, 85),    // #FF5555 Vermelho Brilhante
                RgbColor::new(255, 255, 85),   // #FFFF55 Amarelo Brilhante
            ],
        )
    }

    pub fn pop_art_mac() -> Self {
        Self::new(
            "Old Mac Pop Art",
            vec![
                RgbColor::new(0, 0, 0),        // #000000 Preto
                RgbColor::new(255, 255, 255),  // #FFFFFF Branco
                RgbColor::new(255, 0, 127),    // #FF007F Rosa Pop Art
                RgbColor::new(255, 230, 0),    // #FFE600 Amarelo Serigráfico
                RgbColor::new(0, 210, 255),    // #00D2FF Azul Turquesa Amiga
            ],
        )
    }

    pub fn brutalismo() -> Self {
        Self::new(
            "Brutalismo Concreto (Drica D'arc Meirelles)",
            vec![
                RgbColor::new(10, 10, 10),      // #0A0A0A Preto Piche
                RgbColor::new(229, 229, 229),   // #E5E5E5 Fundo Concreto
                RgbColor::new(250, 250, 250),   // #FAFAFA Branco Gesso
            ],
        )
    }

    pub fn trash_polka() -> Self {
        Self::new(
            "Trash Polka / Realismo & Sangue (Renata Carvalho)",
            vec![
                RgbColor::new(17, 17, 17),      // #111111 Preto Carvão
                RgbColor::new(122, 122, 122),   // #7A7A7A Cinza Meio-Tom
                RgbColor::new(220, 220, 220),   // #DCDCDC Cinza Claro
                RgbColor::new(255, 255, 255),   // #FFFFFF Branco Cirúrgico
                RgbColor::new(186, 12, 47),     // #BA0C2F Vermelho Sangue Coagulado (Carmim)
            ],
        )
    }

    pub fn atari_gaming() -> Self {
        // Tabela NTSC Stella TIA oficial (16 matizes x 8 luminâncias = 128 cores)
        let tia_hex = [
            // Hue 0 (Cinzas)
            "000000", "404040", "6C6C6C", "909090", "B0B0B0", "C8C8C8", "DCDCDC", "ECECEC",
            // Hue 1 (Ouro)
            "444400", "646410", "848424", "A0A034", "B8B840", "D0D050", "E8E860", "FCFC70",
            // Hue 2 (Laranja Ocre)
            "702800", "844414", "985C28", "AC703C", "C08850", "D09C64", "E0B078", "ECC488",
            // Hue 3 (Vermelho Laranja)
            "841800", "983418", "AC4C2C", "C06040", "D07454", "DC8868", "E89C7C", "F0B090",
            // Hue 4 (Vermelho Magenta)
            "880000", "9C2020", "B03C3C", "C05454", "D06C6C", "DC8080", "E89494", "F0A8A8",
            // Hue 5 (Púrpura)
            "78005C", "8C2074", "A03C88", "B0549C", "C06CAC", "CC80BC", "D894CC", "E4A8DC",
            // Hue 6 (Violeta)
            "480078", "602090", "783CA4", "8C54B8", "A06CC8", "B480D8", "C494E4", "D4A8F0",
            // Hue 7 (Azul Cobalto)
            "140084", "302098", "4C3CAC", "6854C0", "7C6CD0", "9080DC", "A494E8", "B8A8F0",
            // Hue 8 (Azul Puro)
            "000088", "1C209C", "3840B0", "505C20", "6874D0", "7C8CDC", "90A0E8", "A4B4F0",
            // Hue 9 (Azul Celeste)
            "00187C", "1C3890", "3854A8", "5070BC", "6888CC", "7C9CDC", "90B0EC", "A4C4F4",
            // Hue 10 (Ciano Turquesa)
            "002C5C", "184878", "306494", "4880AC", "6098C0", "78B0D4", "8CC4E4", "A0D8F4",
            // Hue 11 (Ciano / Aqua)
            "003C2C", "185848", "307464", "489080", "60AC98", "78C4B0", "8CDC24", "A0F0C8",
            // Hue 12 (Verde)
            "003C00", "185818", "307430", "489048", "60AC60", "78C478", "8CDC8C", "A0F0A0",
            // Hue 13 (Verde Amarelado)
            "143800", "2C5414", "447028", "5C8C3C", "74A450", "8CBC64", "A4D078", "BCE488",
            // Hue 14 (Oliva / Musgo)
            "2C3000", "444C10", "5C6824", "748034", "8C9844", "A4B054", "BCC464", "D4D874",
            // Hue 15 (Marrom Quente)
            "402800", "583C10", "705020", "886830", "A08040", "B89450", "D0AC60", "E4C070",
        ];
        let colors = tia_hex
            .iter()
            .map(|h| RgbColor::from_hex(h).unwrap())
            .collect();
        Self::new("Atari 2600 TIA Stella (Jovanna Baby)", colors)
    }

    pub fn mimeografo() -> Self {
        Self::new(
            "Mimeógrafo Violeta (Keila Simpson)",
            vec![
                RgbColor::new(244, 239, 230),   // #F4EFE6 Papel Manilha/Palha
                RgbColor::new(67, 44, 122),     // #432C7A Tinta Violeta Metil
                RgbColor::new(42, 25, 84),      // #2A1954 Saturação de Álcool
            ],
        )
    }

    pub fn xilogravura() -> Self {
        Self::new(
            "Xilogravura Sertaneja (Xica Manicongo)",
            vec![
                RgbColor::new(214, 196, 165),   // #D6C4A5 Papel Kraft Queimado
                RgbColor::new(19, 17, 16),      // #131110 Preto de Fumo
            ],
        )
    }

    pub fn glitch_crt() -> Self {
        Self::new(
            "Glitch CRT & Contrainformação (Robeyoncé Lima)",
            vec![
                RgbColor::new(13, 15, 18),      // #0D0F12 Fundo Ruído Escuro
                RgbColor::new(0, 255, 204),     // #00FFCC Fósforo P31 Verde/Ciano
                RgbColor::new(255, 0, 51),      // #FF0033 Aberração Vermelha
                RgbColor::new(0, 51, 255),      // #0033FF Aberração Azul
            ],
        )
    }

    pub fn terminal_green() -> Self {
        Self::new(
            "Terminal Fósforo Verde (ASCII 1971)",
            vec![
                RgbColor::new(5, 10, 5),       // #050A05 Fundo CRT escuro
                RgbColor::new(0, 85, 20),      // #005514 Verde escuro
                RgbColor::new(0, 170, 50),     // #00AA32 Verde médio
                RgbColor::new(0, 255, 102),    // #00FF66 Fósforo verde brilhante
            ],
        )
    }

    pub fn tabloide_popular() -> Self {
        Self::new(
            "Tabloide Popular",
            vec![
                RgbColor::new(10, 10, 10),      // #0A0A0A Preto chumbo rotativa
                RgbColor::new(230, 0, 0),       // #E60000 Vermelho manchete
                RgbColor::new(255, 230, 0),     // #FFE600 Amarelo sensacionalista
                RgbColor::new(232, 230, 223),   // #E8E6DF Papel jornal cinza
            ],
        )
    }

    pub fn linoleo_ouro() -> Self {
        Self::new(
            "Linóleo & Ouro Queimado (Madame Satã)",
            vec![
                RgbColor::new(26, 20, 18),      // #1A1412 Carvão linóleo
                RgbColor::new(212, 136, 58),    // #D4883A Ouro cru queimado
                RgbColor::new(255, 255, 255),   // #FFFFFF Branco navalha
            ],
        )
    }

    pub fn kraft_olivetti() -> Self {
        Self::new(
            "Papel Kraft & Datilografia Olivetti (João W. Nery)",
            vec![
                RgbColor::new(43, 29, 12),      // #2B1D0C Sépia carbono máquina
                RgbColor::new(200, 168, 112),   // #C8A870 Papel kraft envelhecido
                RgbColor::new(245, 235, 225),   // #F5EBE1 Papel datilografado
            ],
        )
    }

    pub fn teatro_operario() -> Self {
        Self::new(
            "Teatro Operário Construtivista (Leo Moreira Sá)",
            vec![
                RgbColor::new(17, 17, 17),      // #111111 Preto chumbo
                RgbColor::new(179, 27, 27),     // #B31B1B Vermelho carmesim
                RgbColor::new(230, 226, 216),   // #E6E2D8 Papel jornal cru
            ],
        )
    }

    pub fn cyber_bbs() -> Self {
        Self::new(
            "Cyber-BBS Fanzine Digital (Amara Moira)",
            vec![
                RgbColor::new(0, 17, 136),      // #001188 Azul ultramarino BBS
                RgbColor::new(255, 238, 0),     // #FFEE00 Amarelo fósforo CRT
                RgbColor::new(255, 255, 255),   // #FFFFFF Branco puro
            ],
        )
    }

    pub fn punk_80_br() -> Self {
        Self::new(
            "Punk 80 BR // ABC Paulista & K7 (Marina Mathey)",
            vec![
                RgbColor::new(20, 20, 20),      // #141414 Preto asfalto / piche fabril
                RgbColor::new(166, 43, 43),     // #A62B2B Vermelho óxido / ferrugem
                RgbColor::new(230, 226, 212),   // #E6E2D4 Branco papel K7 gasto
            ],
        )
    }

    pub fn anarco_zine() -> Self {
        Self::new(
            "Anarco-Zine // Stencil & Grampos (Jaqueline Gomes de Jesus)",
            vec![
                RgbColor::new(10, 10, 10),      // #0A0A0A Preto anarquia puro
                RgbColor::new(212, 28, 28),     // #D41C1C Vermelho fogo libertário
                RgbColor::new(214, 206, 189),   // #D6CEBD Papel kraft / jornal artesanal
            ],
        )
    }

    pub fn comunicacao_popular() -> Self {
        Self::new(
            "Comunicação Popular // Linogravura & Voz das Ruas (Paulete Furacão)",
            vec![
                RgbColor::new(26, 20, 18),      // #1A1412 Preto Nanquim / Carvão de rua
                RgbColor::new(246, 240, 226),   // #F6F0E2 Papel Panfleto / Kraft Claro
                RgbColor::new(217, 56, 30),     // #D9381E Vermelho Terra / Luta Popular
                RgbColor::new(245, 183, 0),     // #F5B700 Amarelo Dendê / Sol Baiano
            ],
        )
    }

    pub fn imprensa_amazonica() -> Self {
        Self::new(
            "Imprensa Amazônica // Retícula Viva & Resistência (Symmy Larrat)",
            vec![
                RgbColor::new(13, 20, 18),      // #0D1412 Preto Floresta Profunda
                RgbColor::new(242, 237, 228),   // #F2EDE4 Papel Tabloide Ribeirinho
                RgbColor::new(19, 75, 54),      // #134B36 Verde Clorofila / Florestal
                RgbColor::new(255, 0, 85),      // #FF0055 Magenta Choque da Resistência
            ],
        )
    }

    pub fn all_presets() -> Vec<Self> {
        vec![
            Self::classic_punk(),
            Self::linoleo_ouro(),
            Self::xerox_podre(),
            Self::kraft_olivetti(),
            Self::lapa_noturna(),
            Self::teatro_operario(),
            Self::risografia_crua(),
            Self::cyber_bbs(),
            Self::asfalto_ouro(),
            Self::vga_256(),
            Self::cga_palette1(),
            Self::pop_art_mac(),
            Self::terminal_green(),
            Self::tabloide_popular(),
            Self::brutalismo(),
            Self::trash_polka(),
            Self::atari_gaming(),
            Self::mimeografo(),
            Self::xilogravura(),
            Self::glitch_crt(),
            Self::punk_80_br(),
            Self::anarco_zine(),
            Self::comunicacao_popular(),
            Self::imprensa_amazonica(),
        ]
    }

    pub fn find_preset(name_or_id: &str) -> Option<Self> {
        let norm = name_or_id.to_lowercase().replace(['-', '_', ' '], "");
        match norm.as_str() {
            "classicpunk" | "punkriot" | "navalha" | "punk" | "claudiawonder" => Some(Self::classic_punk()),
            "linoleoouro" | "navalhasata" | "sata" | "madamesata" => Some(Self::linoleo_ouro()),
            "xeroxpodre" | "xerox" | "indianaraesiqueira" | "indianarae" => Some(Self::xerox_podre()),
            "kraftolivetti" | "viagemsolitaria" | "nery" | "joaownery" => Some(Self::kraft_olivetti()),
            "lapanoturna" | "lapa" | "neon" | "linndaquebrada" | "linn" => Some(Self::lapa_noturna()),
            "teatrooperario" | "leosa" | "leomoreirasa" | "construtivista" => Some(Self::teatro_operario()),
            "risografiacrua" | "risografia" | "riso" | "erikahilton" => Some(Self::risografia_crua()),
            "cyberbbs" | "fanzinedigital" | "amaramoir" | "amara" | "amramoira" | "bbs" => Some(Self::cyber_bbs()),
            "asfaltoeouro" | "asfaltoalerta" | "asfalto" | "ouro" | "katiatapety" => Some(Self::asfalto_ouro()),
            "vga256" | "vga" | "mode13h" | "marciapantera" => Some(Self::vga_256()),
            "cga" | "cga1" | "cgapalette1" | "neoncunha" => Some(Self::cga_palette1()),
            "cga0" | "cgapalette0" => Some(Self::cga_palette0()),
            "oldmacpopart" | "oldmac" | "popart" | "warhol" | "rogeria" => Some(Self::pop_art_mac()),
            "terminalgreen" | "ascii" | "asciiart" | "cordeiro" | "laerte" | "laertecoutinho" => Some(Self::terminal_green()),
            "tabloidepopular" | "tabloide" | "noticiaspopulares" | "imprensa" | "luanamuniz" => Some(Self::tabloide_popular()),
            "brutalismo" | "brutalista" | "dricadarc" | "dricadarcmeirelles" => Some(Self::brutalismo()),
            "trashpolka" | "trash" | "polka" | "renatacarvalho" => Some(Self::trash_polka()),
            "atarigaming" | "atari" | "tia" | "stella" | "jovannababy" => Some(Self::atari_gaming()),
            "mimeografo" | "mimeo" | "anilina" | "keilasimpson" | "keila" => Some(Self::mimeografo()),
            "xilogravura" | "xilo" | "cordel" | "xicamanicongo" => Some(Self::xilogravura()),
            "glitchcrt" | "glitch" | "vhs" | "robeyonce" | "robeyoncelima" => Some(Self::glitch_crt()),
            "punk80br" | "punk80" | "abc" | "k7" | "marinamathey" | "marina" => Some(Self::punk_80_br()),
            "anarcozine" | "anarco" | "zine" | "stencil" | "jaqueline" | "jaquelinejesus" | "jaquelinegomesdejesus" => Some(Self::anarco_zine()),
            "comunicacaopopular" | "pauletefuracao" | "paulete" | "furacao" => Some(Self::comunicacao_popular()),
            "imprensaamazonica" | "symmylarrat" | "symmy" | "larrat" | "amazonia" => Some(Self::imprensa_amazonica()),
            _ => None,
        }
    }
}
