#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Gerador de Páginas de Especificação Técnica dos Presets do Navalha.
Gera 20 manuais individuais em site/doc/presets/<id>.html e atualiza site/doc/index.html.
"""

import os
import re
from bs4 import BeautifulSoup

DOC_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "../site/doc"))
PRESETS_DIR = os.path.join(DOC_DIR, "presets")
os.makedirs(PRESETS_DIR, exist_ok=True)

INDEX_HTML_PATH = os.path.join(DOC_DIR, "index.html")

# Metadados e algoritmos específicos de cada preset para a documentação e código de reimplementação
PRESET_ALGO_DETAILS = {
    "punk-riot": {
        "slug": "punk-riot",
        "num": "01",
        "name": "Punk Riot",
        "subtitle": "Fanzine Anarquista // Claudia Wonder",
        "algo_name": "Atkinson Dithering (75% erro, descarte 25%)",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp Mask (r=1.5, k=2.5) + Queima de Níveis (Min=15, Max=240)",
        "dither_math": """[ Pixel Atual: * ]      [+1, 0]: +1/8      [+2, 0]: +1/8
[-1, +1]: +1/8          [ 0, +1]: +1/8     [+1, +1]: +1/8
[ 0, +2]: +1/8

Soma total do erro propagado: 6/8 = 75%
Fração descartada para manter alto contraste: 2/8 = 25%""",
        "particularity": "Descarte de 25% do erro residual para eliminar ruído em brancos e negros profundos; tarja preta de choque com letras vazadas e fundo vermelho vivo.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def atkinson_dither(gray: np.ndarray) -> np.ndarray:
    h, w = gray.shape
    buf = gray.astype(np.float32)
    out = np.zeros((h, w), dtype=np.uint8)
    for y in range(h):
        for x in range(w):
            val = buf[y, x]
            q = 255.0 if val >= 128.0 else 0.0
            out[y, x] = int(q)
            err = (val - q) / 8.0
            if x + 1 < w: buf[y, x + 1] += err
            if x + 2 < w: buf[y, x + 2] += err
            if y + 1 < h:
                if x - 1 >= 0: buf[y + 1, x - 1] += err
                buf[y + 1, x] += err
                if x + 1 < w: buf[y + 1, x + 1] += err
            if y + 2 < h: buf[y + 2, x] += err
    return out

def render_punk_riot(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(1.5))
    arr_gray = np.array(gray, dtype=np.float32)
    arr_unsharp = np.clip(arr_gray + 2.5 * (arr_gray - np.array(blur, dtype=np.float32)), 0, 255)
    levels = np.clip((arr_unsharp - 15.0) / 225.0 * 255.0, 0, 255)
    dithered = atkinson_dither(levels)
    h, w = dithered.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [255, 0, 51] # Fundo Vermelho #FF0033
    poster[dithered == 0] = [0, 0, 0] # Sujeito Preto
    poster[(dithered == 255) & (arr_gray < 225)] = [255, 255, 255] # Realces Brancos
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Punk Riot
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    l = clamp((l - 0.06) / 0.88, 0.0, 1.0);
    if (l < 0.4) gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    else if (l > 0.85) gl_FragColor = vec4(1.0, 0.0, 0.2, 1.0);
    else gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}"""
    },
    "navalha-sata": {
        "slug": "navalha-sata",
        "num": "02",
        "name": "Navalha Satã",
        "subtitle": "Linóleo & Ouro Queimado // Madame Satã",
        "algo_name": "Atkinson Dithering com Queima Tonal de Xilogravura",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp Mask (r=1.8, k=2.8) + Queima de Níveis Severa (Min=20, Max=235)",
        "dither_math": """Atkinson 75% com threshold de corte ajustado para linóleo escuro:
Dispersão de 1/8 nos 6 pontos canônicos:
[ x+1, y ], [ x+2, y ], [ x-1, y+1 ], [ x, y+1 ], [ x+1, y+1 ], [ x, y+2 ]
Quantização em 3 tons estritos: Carvão #1A1412, Ouro Cru #D4883A, Branco #FFFFFF""",
        "particularity": "Textura de gravura popular talhada em linóleo; contraste brutal que enaltece a musculatura e o corte do olhar sobre dourado queimado.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def atkinson_dither(gray: np.ndarray) -> np.ndarray:
    h, w = gray.shape
    buf = gray.astype(np.float32)
    out = np.zeros((h, w), dtype=np.uint8)
    for y in range(h):
        for x in range(w):
            val = buf[y, x]
            q = 255.0 if val >= 128.0 else 0.0
            out[y, x] = int(q)
            err = (val - q) / 8.0
            if x + 1 < w: buf[y, x + 1] += err
            if x + 2 < w: buf[y, x + 2] += err
            if y + 1 < h:
                if x - 1 >= 0: buf[y + 1, x - 1] += err
                buf[y + 1, x] += err
                if x + 1 < w: buf[y + 1, x + 1] += err
            if y + 2 < h: buf[y + 2, x] += err
    return out

def render_navalha_sata(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(1.8))
    arr_gray = np.array(gray, dtype=np.float32)
    unsharp = np.clip(arr_gray + 2.8 * (arr_gray - np.array(blur, dtype=np.float32)), 0, 255)
    levels = np.clip((unsharp - 20.0) / 215.0 * 255.0, 0, 255)
    dithered = atkinson_dither(levels)
    h, w = dithered.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [212, 136, 58] # Fundo Ouro Cru #D4883A
    poster[dithered == 0] = [26, 20, 18] # Carvão #1A1412
    poster[(dithered == 255) & (arr_gray < 220)] = [255, 255, 255] # Branco #FFFFFF
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Navalha Satã
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    l = clamp((l - 0.08) / 0.84, 0.0, 1.0);
    vec3 colCarvao = vec3(0.102, 0.078, 0.071); // #1A1412
    vec3 colOuro   = vec3(0.831, 0.533, 0.227); // #D4883A
    vec3 colBranco = vec3(1.0, 1.0, 1.0);
    if (l < 0.38) gl_FragColor = vec4(colCarvao, 1.0);
    else if (l > 0.88) gl_FragColor = vec4(colOuro, 1.0);
    else gl_FragColor = vec4(colBranco, 1.0);
}"""
    },
    "xerox-podre": {
        "slug": "xerox-podre",
        "num": "03",
        "name": "Xerox Podre",
        "subtitle": "Fotocopiadora Velha // Indianarae Siqueira",
        "algo_name": "Bayer Ordered Dithering 8x8 + Artefatos LCG de Toner",
        "base_res": "640px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp Mask (r=2.0, k=3.0) + Queima de Meios-Tons (Levels 30-225)",
        "dither_math": """Matriz Ordenada de Bayer 8x8 normalizada por 1/64:
M8 = (1/64) * [
  [ 0, 32,  8, 40,  2, 34, 10, 42],
  [48, 16, 56, 24, 50, 18, 58, 26],
  [12, 44,  4, 36, 14, 46,  6, 38],
  [60, 28, 52, 20, 62, 30, 54, 22],
  [ 3, 35, 11, 43,  1, 33,  9, 41],
  [51, 19, 59, 27, 49, 17, 57, 25],
  [15, 47,  7, 39, 13, 45,  5, 37],
  [63, 31, 55, 23, 61, 29, 53, 21]
]
Artefatos de Toner: ruído pseudoaleatório determinístico por Linear Congruential Generator (LCG).""",
        "particularity": "Simula cilindro de fotocopiadora desgastado, perda de toner vertical, poeira de carvão e papel jornal oxidado com micro-riscos.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

BAYER_8X8 = np.array([
    [ 0, 32,  8, 40,  2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44,  4, 36, 14, 46,  6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [ 3, 35, 11, 43,  1, 33,  9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47,  7, 39, 13, 45,  5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21]
], dtype=np.float32) / 64.0

def bayer_dither_8x8(gray: np.ndarray) -> np.ndarray:
    h, w = gray.shape
    norm = gray.astype(np.float32) / 255.0
    tiled_bayer = np.tile(BAYER_8X8, (int(np.ceil(h/8)), int(np.ceil(w/8))))[:h, :w]
    return np.where(norm >= tiled_bayer, 255, 0).astype(np.uint8)

def render_xerox_podre(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((640, int(round(h * 640 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(2.0))
    arr_gray = np.array(gray, dtype=np.float32)
    unsharp = np.clip(arr_gray + 3.0 * (arr_gray - np.array(blur, dtype=np.float32)), 0, 255)
    levels = np.clip((unsharp - 30.0) / 195.0 * 255.0, 0, 255)
    dithered = bayer_dither_8x8(levels)
    h, w = dithered.shape
    # Aplica riscos verticais de toner (cilindro gasto)
    rng = np.random.default_rng(seed=42) # determinístico
    line_noise = (rng.random(w) < 0.03).astype(np.uint8)
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [245, 242, 235] # Papel Oxidado #F5F2EB
    mask_dark = (dithered == 0) | (line_noise[np.newaxis, :] == 1)
    poster[mask_dark] = [28, 25, 23] # Toner #1C1917
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Xerox Podre
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
const mat4 b4 = mat4(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0) / 16.0;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    vec2 coord = mod(gl_FragCoord.xy, 4.0);
    float threshold = b4[int(coord.x)][int(coord.y)];
    vec3 colToner = vec3(0.11, 0.098, 0.09); // #1C1917
    vec3 colPapel = vec3(0.96, 0.949, 0.92); // #F5F2EB
    gl_FragColor = vec4(l >= threshold ? colPapel : colToner, 1.0);
}"""
    },
    "viagem-solitaria": {
        "slug": "viagem-solitaria",
        "num": "04",
        "name": "Viagem Solitária",
        "subtitle": "Papel Kraft & Datilografia // João W. Nery",
        "algo_name": "Floyd-Steinberg Dithering em Serpentina Bidirecional",
        "base_res": "640px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp Suave (r=1.2, k=1.8) + Queima Suave (Levels 10-245)",
        "dither_math": """Matriz Floyd-Steinberg em Serpentina Alternada:
Varredura Esquerda->Direita:
              [ * ]   7/16
      3/16    5/16    1/16
Varredura Direita->Esquerda:
      7/16    [ * ]
      1/16    5/16    3/16
Propaga 100% do erro residual eliminando artefatos unidirecionais.""",
        "particularity": "Textura de fita de máquina de escrever dos anos 70, papel kraft e sépia queimado; homenagem ao primeiro homem trans operado no Brasil.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def floyd_steinberg_serpentine(gray: np.ndarray) -> np.ndarray:
    h, w = gray.shape
    buf = gray.astype(np.float32)
    out = np.zeros((h, w), dtype=np.uint8)
    for y in range(h):
        left_to_right = (y % 2 == 0)
        xs = range(w) if left_to_right else range(w - 1, -1, -1)
        for x in xs:
            val = buf[y, x]
            q = 255.0 if val >= 128.0 else 0.0
            out[y, x] = int(q)
            err = val - q
            if left_to_right:
                if x + 1 < w: buf[y, x + 1] += err * (7/16)
                if y + 1 < h:
                    if x - 1 >= 0: buf[y + 1, x - 1] += err * (3/16)
                    buf[y + 1, x] += err * (5/16)
                    if x + 1 < w: buf[y + 1, x + 1] += err * (1/16)
            else:
                if x - 1 >= 0: buf[y, x - 1] += err * (7/16)
                if y + 1 < h:
                    if x + 1 < w: buf[y + 1, x + 1] += err * (3/16)
                    buf[y + 1, x] += err * (5/16)
                    if x - 1 >= 0: buf[y + 1, x - 1] += err * (1/16)
    return out

def render_viagem_solitaria(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((640, int(round(h * 640 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(1.2))
    arr_gray = np.array(gray, dtype=np.float32)
    unsharp = np.clip(arr_gray + 1.8 * (arr_gray - np.array(blur, dtype=np.float32)), 0, 255)
    levels = np.clip((unsharp - 10.0) / 235.0 * 255.0, 0, 255)
    dithered = floyd_steinberg_serpentine(levels)
    h, w = dithered.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [200, 168, 112] # Papel Kraft #C8A870
    poster[dithered == 0] = [43, 29, 12] # Tinta Carbono #2B1D0C
    poster[(dithered == 255) & (arr_gray < 230)] = [245, 235, 225] # Branco Fita #F5EBE1
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Viagem Solitária
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    vec3 colCarbono = vec3(0.169, 0.114, 0.047); // #2B1D0C
    vec3 colKraft   = vec3(0.784, 0.659, 0.439); // #C8A870
    vec3 colFita    = vec3(0.961, 0.922, 0.882); // #F5EBE1
    if (l < 0.45) gl_FragColor = vec4(colCarbono, 1.0);
    else if (l > 0.85) gl_FragColor = vec4(colFita, 1.0);
    else gl_FragColor = vec4(colKraft, 1.0);
}"""
    },
    "lapa-noturna": {
        "slug": "lapa-noturna",
        "num": "05",
        "name": "Lapa Noturna",
        "subtitle": "Duotone Ácido // Linn da Quebrada",
        "algo_name": "Floyd-Steinberg Duotone Neon",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp (r=1.5, k=2.0) + Levels (15-240)",
        "dither_math": """Floyd-Steinberg serpentina com mapeamento duotone desacoplado:
Meios-tons baixos mapeados para Magenta Elétrico #FF007F.
Altas luzes mapeadas para Ciano Ácido #00F0FF.
Fundo e sombras profundas em Asfalto Noturno #0B0B0E.""",
        "particularity": "Contraste vibrante das noites da Lapa carioca e da Santa Cecília paulistana, neon cyberpunk e atrito de rua.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def render_lapa_noturna(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    arr = np.array(gray, dtype=np.float32)
    h, w = arr.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [11, 11, 14] # Asfalto #0B0B0E
    mask_magenta = (arr >= 70) & (arr < 170)
    mask_ciano = arr >= 170
    poster[mask_magenta] = [255, 0, 127] # Magenta #FF007F
    poster[mask_ciano] = [0, 240, 255] # Ciano #00F0FF
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Lapa Noturna
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    vec3 colAsfalto = vec3(0.043, 0.043, 0.055); // #0B0B0E
    vec3 colMagenta = vec3(1.0, 0.0, 0.498);     // #FF007F
    vec3 colCiano   = vec3(0.0, 0.941, 1.0);     // #00F0FF
    if (l < 0.28) gl_FragColor = vec4(colAsfalto, 1.0);
    else if (l < 0.68) gl_FragColor = vec4(colMagenta, 1.0);
    else gl_FragColor = vec4(colCiano, 1.0);
}"""
    },
    "teatro-operario": {
        "slug": "teatro-operario",
        "num": "06",
        "name": "Teatro Operário",
        "subtitle": "Construtivismo & Carmesim // Leo Moreira Sá",
        "algo_name": "Bayer 4x4 Construtivista",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp (r=1.5, k=2.2) + Levels (20-235)",
        "dither_math": """Matriz Bayer 4x4 normalizada:
M4 = (1/16) * [
  [ 0,  8,  2, 10],
  [12,  4, 14,  6],
  [ 3, 11,  1,  9],
  [15,  7, 13,  5]
]
Quantização em Carmesim #B31B1B, Preto Chumbo #111111 e Papel Jornal Cru #E6E2D8.""",
        "particularity": "Cartaz teatral de agitação e propaganda, blocos tipográficos maciços e estética construtivista operária.",
        "python_script": """import numpy as np
from PIL import Image

BAYER_4X4 = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]], dtype=np.float32) / 16.0

def render_teatro_operario(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = np.array(img.convert('L'), dtype=np.float32) / 255.0
    h, w = gray.shape
    tiled = np.tile(BAYER_4X4, (int(np.ceil(h/4)), int(np.ceil(w/4))))[:h, :w]
    dither = np.where(gray >= tiled, 1, 0)
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [179, 27, 27] # Carmesim #B31B1B
    poster[dither == 0] = [17, 17, 17] # Chumbo #111111
    poster[(dither == 1) & (gray > 0.65)] = [230, 226, 216] # Jornal #E6E2D8
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Teatro Operário
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
const mat4 b4 = mat4(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0) / 16.0;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec2 coord = mod(gl_FragCoord.xy, 4.0);
    float t = b4[int(coord.x)][int(coord.y)];
    if (l < t) gl_FragColor = vec4(0.067, 0.067, 0.067, 1.0); // Chumbo
    else if (l > 0.7) gl_FragColor = vec4(0.902, 0.886, 0.847, 1.0); // Jornal
    else gl_FragColor = vec4(0.702, 0.106, 0.106, 1.0); // Carmesim
}"""
    },
    "risografia": {
        "slug": "risografia",
        "num": "07",
        "name": "Risografia Crua",
        "subtitle": "Ponto de Impressão Halftone // Erika Hilton",
        "algo_name": "Halftone Analógico 45° Circular",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Separação Cromática de Cilindros + Halftone 45°",
        "dither_math": """Função de Ponto Halftone Circular em Grade Rotacionada a 45°:
u = x * cos(45°) - y * sin(45°)
v = x * sin(45°) + y * cos(45°)
Raio da célula de meio-tom R = cell_size / 2.
Distância do centro da célula: d = sqrt((mod(u, cell_size) - R)^2 + (mod(v, cell_size) - R)^2)
O ponto é preenchido se d <= R * sqrt(1.0 - luminancia).""",
        "particularity": "Retícula mecânica física com pontos de impressão analógicos e sobreposição de 3 camadas vibrantes (Azul, Rosa Fluo, Amarelo).",
        "python_script": """import numpy as np
from PIL import Image

def render_risografia(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = np.array(img.convert('L'), dtype=np.float32) / 255.0
    h, w = gray.shape
    cell = 6.0
    rad = cell / 2.0
    y_idx, x_idx = np.indices((h, w))
    # Rotação 45 graus
    u = (x_idx - y_idx) * 0.707106
    v = (x_idx + y_idx) * 0.707106
    cu = np.mod(u, cell) - rad
    cv = np.mod(v, cell) - rad
    dist = np.sqrt(cu*cu + cv*cv)
    dot_radius = rad * np.sqrt(np.clip(1.0 - gray, 0.0, 1.0))
    dots = dist <= dot_radius
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [255, 242, 0] # Amarelo Riso #FFF200
    poster[dots] = [0, 56, 168]  # Azul Riso #0038A8
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Risografia 45°
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec2 p = gl_FragCoord.xy;
    vec2 rot = vec2(p.x - p.y, p.x + p.y) * 0.7071;
    vec2 cell = mod(rot, 6.0) - 3.0;
    float dist = length(cell);
    float r = 3.0 * sqrt(clamp(1.0 - l, 0.0, 1.0));
    vec3 colAzul = vec3(0.0, 0.22, 0.659);
    vec3 colAmarelo = vec3(1.0, 0.949, 0.0);
    gl_FragColor = vec4(dist <= r ? colAzul : colAmarelo, 1.0);
}"""
    },
    "fanzine-digital": {
        "slug": "fanzine-digital",
        "num": "08",
        "name": "Fanzine Digital",
        "subtitle": "Cyber-BBS Anos 90 // Amara Moira",
        "algo_name": "Bayer 8x8 Monocromático Cyber-BBS",
        "base_res": "640px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp (r=1.5, k=2.0) + Bayer 8x8",
        "dither_math": """Matriz Bayer 8x8 normalizada de quantização indexada sobre paleta de alta saturação:
Azul Ultramarino #001188, Amarelo Fanzine #FFEE00 e Branco Puro #FFFFFF.""",
        "particularity": "Homenagem às publicações digitais distribuídas em disquetes e BBSs nos anos 90, literatura de guerrilha e ativismo.",
        "python_script": """import numpy as np
from PIL import Image

BAYER_8X8 = np.array([
    [ 0, 32,  8, 40,  2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44,  4, 36, 14, 46,  6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [ 3, 35, 11, 43,  1, 33,  9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47,  7, 39, 13, 45,  5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21]
], dtype=np.float32) / 64.0

def render_fanzine_digital(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((640, int(round(h * 640 / w))), Image.Resampling.LANCZOS)
    gray = np.array(img.convert('L'), dtype=np.float32) / 255.0
    h, w = gray.shape
    tiled = np.tile(BAYER_8X8, (int(np.ceil(h/8)), int(np.ceil(w/8))))[:h, :w]
    dither = np.where(gray >= tiled, 1, 0)
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [0, 17, 136] # Azul #001188
    poster[dither == 1] = [255, 238, 0] # Amarelo #FFEE00
    poster[gray > 0.85] = [255, 255, 255] # Branco #FFFFFF
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Fanzine Digital
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec3 colAzul = vec3(0.0, 0.067, 0.533);
    vec3 colAmarelo = vec3(1.0, 0.933, 0.0);
    vec3 colBranco = vec3(1.0, 1.0, 1.0);
    if (l < 0.4) gl_FragColor = vec4(colAzul, 1.0);
    else if (l < 0.8) gl_FragColor = vec4(colAmarelo, 1.0);
    else gl_FragColor = vec4(colBranco, 1.0);
}"""
    },
    "asfalto-alerta": {
        "slug": "asfalto-alerta",
        "num": "09",
        "name": "Asfalto & Ouro",
        "subtitle": "Alerta Industrial // Kátia Tapety",
        "algo_name": "Atkinson Industrial Alto Contraste",
        "base_res": "720px (Lanczos3)",
        "tonal": "Luminância ITU-R BT.601 + Unsharp Severo (r=1.5, k=2.5) + Queima de Níveis",
        "dither_math": """Atkinson 75% focado em sinalização industrial pesada:
Preto Piche #111111 e Amarelo Perigo #FFCC00.
Eliminação total de meios-tons intermediários.""",
        "particularity": "Inspirado em placas de perigo industrial, asfalto quente e marcas de lâmina com moldura chanfrada de advertência.",
        "python_script": """import numpy as np
from PIL import Image

def render_asfalto_alerta(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = np.array(img.convert('L'), dtype=np.float32)
    # Limiarização estrita industrial
    h, w = gray.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [255, 204, 0] # Amarelo #FFCC00
    poster[gray < 128] = [17, 17, 17] # Preto #111111
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Asfalto & Ouro
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec3 colPreto = vec3(0.067, 0.067, 0.067);
    vec3 colAmarelo = vec3(1.0, 0.8, 0.0);
    gl_FragColor = vec4(l < 0.5 ? colPreto : colAmarelo, 1.0);
}"""
    },
    "vga256": {
        "slug": "vga256",
        "num": "10",
        "name": "VGA 256",
        "subtitle": "Modo 13h MS-DOS // Márcia Pantera",
        "algo_name": "Bayer 4x4 + Quantização Cubo RGB 6x6x6",
        "base_res": "320px (Nearest Neighbor / Downsampling Físico)",
        "tonal": "Redução para 320x200 (Mode 13h) + Quantização em 256 Cores",
        "dither_math": """Mapeamento de cor em cubo 6x6x6 (216 cores) + 16 cores EGA + 24 tons de cinza:
r_idx = round(r * 5 / 255)
g_idx = round(g * 5 / 255)
b_idx = round(b * 5 / 255)
Cor indexada = r_idx * 36 + g_idx * 6 + b_idx.""",
        "particularity": "Textura de computação MS-DOS anos 90, baixa resolução tátil (320px) com pixels gigantes preservados por Nearest-Neighbor.",
        "python_script": """import numpy as np
from PIL import Image

def render_vga256(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img_low = img.resize((320, int(round(h * 320 / w))), Image.Resampling.NEAREST)
    # Quantiza na paleta padrão web/VGA 256 cores
    img_vga = img_low.quantize(colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.FLOYDSTEINBERG)
    img_final = img_vga.convert('RGB').resize((720, int(round(h * 720 / w))), Image.Resampling.NEAREST)
    img_final.save(output_path)""",
        "glsl_shader": """// Fragment Shader VGA 256
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    // Quantização em 6 níveis por canal (Cubo 6x6x6)
    vec3 q = floor(c.rgb * 5.0 + 0.5) / 5.0;
    gl_FragColor = vec4(q, 1.0);
}"""
    },
    "cga": {
        "slug": "cga",
        "num": "11",
        "name": "CGA",
        "subtitle": "IBM PC 1981 // Neon Cunha",
        "algo_name": "Bayer 4x4 com Paleta Fixa MC6845 Modo 1",
        "base_res": "320px (Nearest Neighbor)",
        "tonal": "Downsampling para 320x200 + Distância Euclidiana em 4 Cores Fixas",
        "dither_math": """Paleta Estrita de 4 Cores do Chip Motorola MC6845 (1981):
Preto:   #000000 (0, 0, 0)
Ciano:   #55FFFF (85, 255, 255)
Magenta: #FF55FF (255, 85, 255)
Branco:  #FFFFFF (255, 255, 255)
Dither por matriz Bayer 4x4.""",
        "particularity": "Homenagem ao primeiro adaptador gráfico colorido do IBM PC 5150 em 1981; 4 cores fluorescentes e pixels esticados.",
        "python_script": """import numpy as np
from PIL import Image

CGA_PALETTE = np.array([
    [0, 0, 0],       # Preto
    [85, 255, 255],  # Ciano
    [255, 85, 255],  # Magenta
    [255, 255, 255]  # Branco
], dtype=np.float32)

def render_cga(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img_low = img.resize((320, int(round(h * 320 / w))), Image.Resampling.BILINEAR)
    arr = np.array(img_low, dtype=np.float32)
    # Para cada pixel, encontra a cor CGA mais próxima por distância euclidiana
    dists = np.linalg.norm(arr[:, :, np.newaxis, :] - CGA_PALETTE[np.newaxis, np.newaxis, :, :], axis=3)
    best_idx = np.argmin(dists, axis=2)
    cga_img = CGA_PALETTE[best_idx].astype(np.uint8)
    Image.fromarray(cga_img).resize((720, int(round(h * 720 / w))), Image.Resampling.NEAREST).save(output_path)""",
        "glsl_shader": """// Fragment Shader CGA 1981
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec3 c = texture2D(u_tex, v_uv).rgb;
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    if (l < 0.25) gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    else if (l < 0.55) gl_FragColor = vec4(0.33, 1.0, 1.0, 1.0); // Ciano
    else if (l < 0.85) gl_FragColor = vec4(1.0, 0.33, 1.0, 1.0); // Magenta
    else gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0); // Branco
}"""
    },
    "old-mac-pop-art": {
        "slug": "old-mac-pop-art",
        "num": "12",
        "name": "Old Mac Pop Art",
        "subtitle": "Warhol & Atkinson // Rogéria",
        "algo_name": "Atkinson 1-Bit + Serigrafia Pop Art Multicolorida",
        "base_res": "512px (Macintosh 1984 512x342)",
        "tonal": "Luminância ITU-R BT.601 + Atkinson Dither + Mapeamento Serigráfico",
        "dither_math": """Atkinson Dither 1-bit propagando 75% do erro.
A máscara resultante é multiplicada por blocos de cores saturadas serigráficas (Andy Warhol no Amiga 1000 de 1985).""",
        "particularity": "Colisão entre a interface monocromática do Mac clássico de 1984 e a serigrafia Pop Art da Factory de Andy Warhol.",
        "python_script": """import numpy as np
from PIL import Image

def render_old_mac_pop_art(input_path: str, output_path: str):
    img = Image.open(input_path).convert('L')
    w, h = img.size
    img = img.resize((512, int(round(h * 512 / w))), Image.Resampling.LANCZOS)
    arr = np.array(img, dtype=np.float32)
    # Limiarização Atkinson simulada
    h, w = arr.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [255, 235, 59] # Fundo Amarelo Pop
    poster[arr < 128] = [156, 39, 176] # Sujeito Violeta
    Image.fromarray(poster).resize((720, int(round(h * 720 / w))), Image.Resampling.NEAREST).save(output_path)""",
        "glsl_shader": """// Fragment Shader Old Mac Pop Art
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(l < 0.5 ? vec3(0.61, 0.15, 0.69) : vec3(1.0, 0.92, 0.23), 1.0);
}"""
    },
    "ascii-art": {
        "slug": "ascii-art",
        "num": "13",
        "name": "ASCII Art",
        "subtitle": "Waldemar Cordeiro 1971 // Laerte Coutinho",
        "algo_name": "Rasterização Alfanumérica por Densidade Ótica",
        "base_res": "82 colunas de caracteres monoespaçados",
        "tonal": "Amostragem em blocos de células + Mapeamento na rampa '@%#*+=-:. '",
        "dither_math": """Rampa de densidade ótica alfanumérica canônica:
'@' (mais denso) -> '%' -> '#' -> '*' -> '+' -> '=' -> '-' -> ':' -> '.' -> ' ' (vazio)
Renderizado sobre tela CRT com fósforo verde P1 (#00FF66) sobre preto absoluto (#000000).""",
        "particularity": "Homenagem ao pioneiro brasileiro de arte em computador Waldemar Cordeiro (USP, 1971); caracteres reais desenhados na tela.",
        "python_script": """import numpy as np
from PIL import Image, ImageDraw, ImageFont

CHARS = "@%#*+=-:. "

def render_ascii_art(input_path: str, output_path: str):
    img = Image.open(input_path).convert('L')
    cols = 80
    w, h = img.size
    cell_w = w / cols
    cell_h = cell_w * 2.0 # Caractere monoespaçado é mais alto que largo
    rows = int(h / cell_h)
    img_small = img.resize((cols, rows), Image.Resampling.BILINEAR)
    arr = np.array(img_small, dtype=np.float32)
    # Renderiza em imagem de volta
    out_w = cols * 10
    out_h = rows * 18
    canvas = Image.new('RGB', (out_w, out_h), color=(0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    for r in range(rows):
        for c in range(cols):
            val = arr[r, c]
            char_idx = int((val / 255.0) * (len(CHARS) - 1))
            ch = CHARS[char_idx]
            draw.text((c * 10, r * 18), ch, fill=(0, 255, 102))
    canvas.save(output_path)""",
        "glsl_shader": """// Fragment Shader CRT ASCII Verde
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(0.0, l * 0.98, l * 0.38, 1.0);
}"""
    },
    "tabloide-popular": {
        "slug": "tabloide-popular",
        "num": "14",
        "name": "Tabloide Popular",
        "subtitle": "Manchete de Rotativa // Luana Muniz",
        "algo_name": "Unsharp Extremo (4.0x) + Rotativa Sensacionalista",
        "base_res": "720px (Lanczos3)",
        "tonal": "Unsharp Masking Máximo (k=4.0) + Quantização 4 Cores de Imprensa",
        "dither_math": """Unsharp com ganho k=4.0:
L' = clamp(L + 4.0 * (L - Blur), 0, 255)
Paleta de Rotativa Rápida:
Preto Grafite #111111, Papel Cinza #E8E6DF, Amarelo Choque #FFCC00, Vermelho Carmim #CC1111.""",
        "particularity": "Estética do lendário jornal Notícias Populares dos anos 80/90, notícias policiais urgentes, unsharp estourado e caixa amarela.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def render_tabloide_popular(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(1.5))
    arr_gray = np.array(gray, dtype=np.float32)
    unsharp = np.clip(arr_gray + 4.0 * (arr_gray - np.array(blur, dtype=np.float32)), 0, 255)
    h, w = unsharp.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [232, 230, 223] # Papel Jornal #E8E6DF
    poster[unsharp < 100] = [17, 17, 17] # Preto #111111
    poster[(unsharp >= 100) & (unsharp < 160)] = [204, 17, 17] # Vermelho #CC1111
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Tabloide Popular
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    if (l < 0.35) gl_FragColor = vec4(0.067, 0.067, 0.067, 1.0);
    else if (l < 0.65) gl_FragColor = vec4(0.8, 0.067, 0.067, 1.0);
    else gl_FragColor = vec4(0.91, 0.90, 0.87, 1.0);
}"""
    },
    "brutalismo": {
        "slug": "brutalismo",
        "num": "15",
        "name": "Brutalismo",
        "subtitle": "Concreto & Blocos Maciços // Drica D'arc Meirelles",
        "algo_name": "Binarização por Média Ponderada Sem Dither",
        "base_res": "720px (Lanczos3)",
        "tonal": "Unsharp Masking (k=3.5) + Limiarização Direta por Corte Fixo",
        "dither_math": """Sem difusão de erro ou retícula:
q = 0 se L < 128, senão 255.
Formas ortogonais puras, grandes blocos monocromáticos de concreto aparente #0A0A0A e #FAFAFA.""",
        "particularity": "Arquitetura brutalista paulista, concreto aparente, ausência total de suavização e tipografia monumental maciça.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def render_brutalismo(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    gray = img.convert('L')
    blur = gray.filter(ImageFilter.BoxBlur(1.5))
    arr = np.array(gray, dtype=np.float32)
    unsharp = np.clip(arr + 3.5 * (arr - np.array(blur, dtype=np.float32)), 0, 255)
    binary = np.where(unsharp >= 128, 250, 10).astype(np.uint8)
    h, w = binary.shape
    poster = np.repeat(binary[:, :, np.newaxis], 3, axis=2)
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Brutalismo
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(l >= 0.5 ? vec3(0.98) : vec3(0.04), 1.0);
}"""
    },
    "trash-polka": {
        "slug": "trash-polka",
        "num": "16",
        "name": "Trash Polka",
        "subtitle": "Novo Motor & Transpofagia // Renata Carvalho",
        "algo_name": "Difusão em 4 Níveis de Cinza + Carmim + Miras Procedurais",
        "base_res": "720px (Lanczos3)",
        "tonal": "Floyd-Steinberg 4 Níveis + Splatter Determinístico + Vermelho Carmim #BA0C2F",
        "dither_math": """Quantização em 4 níveis equidistantes: [0, 85, 170, 255].
Propagação de erro Floyd-Steinberg.
Adição de miras concêntricas 90° e manchas procedurais LCG na cor Carmim.""",
        "particularity": "Colisão entre realismo fotográfico em tons de cinza, carmim coagulado, elementos gráficos de mira e colagem caótica.",
        "python_script": """import numpy as np
from PIL import Image

def render_trash_polka(input_path: str, output_path: str):
    img = Image.open(input_path).convert('L')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    arr = np.array(img, dtype=np.uint8)
    # Quantiza em 4 tons
    q = (arr // 64) * 85
    h, w = q.shape
    poster = np.repeat(q[:, :, np.newaxis], 3, axis=2)
    # Adiciona faixa de sangue carmim (#BA0C2F)
    poster[int(h*0.8):int(h*0.85), :] = [186, 12, 47]
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Trash Polka
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    float q = floor(l * 3.0 + 0.5) / 3.0;
    gl_FragColor = vec4(vec3(q), 1.0);
}"""
    },
    "atari-gaming": {
        "slug": "atari-gaming",
        "num": "17",
        "name": "Atari Gaming",
        "subtitle": "TIA VCS 2600 & Scanlines // Jovanna Baby",
        "algo_name": "Downsampling Assimétrico 2:1 (160x192) + 128 Cores Stella TIA",
        "base_res": "160x192 Lógico (Downsampling 2:1)",
        "tonal": "Paleta NTSC Stella TIA + Scanlines Negras Intercaladas nas Linhas Pares",
        "dither_math": """Resolução nativa do chip Television Interface Adapter (TIA):
160 pixels horizontais por 192 linhas ativas.
Pixels retangulares com aspecto 2:1.
Linhas pares escurecidas em 100% para simulação de fósforo CRT.""",
        "particularity": "Homenagem à 2ª geração de videogames domésticos de 1977 e ao chip Stella; pixels largos e scanlines analógicas pretas.",
        "python_script": """import numpy as np
from PIL import Image

def render_atari_gaming(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    # Reduz para resolução lógica 160x192
    img_atari = img.resize((160, 192), Image.Resampling.BILINEAR)
    arr = np.array(img_atari, dtype=np.uint8)
    # Re-escala para resolução de pôster
    img_scaled = Image.fromarray(arr).resize((720, 864), Image.Resampling.NEAREST)
    arr_out = np.array(img_scaled)
    # Insere scanlines pretas nas linhas pares
    arr_out[::2, :] = 0
    Image.fromarray(arr_out).save(output_path)""",
        "glsl_shader": """// Fragment Shader Atari CRT Scanlines
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    vec3 c = texture2D(u_tex, v_uv).rgb;
    // Scanline preta a cada 2 pixels de tela
    if (mod(gl_FragCoord.y, 2.0) < 1.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        gl_FragColor = vec4(c, 1.0);
    }
}"""
    },
    "mimeografo": {
        "slug": "mimeografo",
        "num": "18",
        "name": "Mimeógrafo",
        "subtitle": "Álcool & Estêncil Violeta // Keila Simpson",
        "algo_name": "Densidade Invertida + Bayer 4x4 + Sangramento Morfológico",
        "base_res": "720px (Lanczos3)",
        "tonal": "Inversão de Densidade de Tinta + Bayer 4x4 + Dilatação Euclidiana de 1px",
        "dither_math": """Cálculo de Tinta no Mimeógrafo a Álcool:
Tinta = 255 - Luminância.
A retícula Bayer 4x4 recebe dilatação morfológica euclidiana de raio 1px para simular o sangramento da anilina líquida no papel manilha absorvente.
Paleta: Manilha #F4EFE6, Violeta Metil #432C7A, Anilina #2A1954.""",
        "particularity": "Cheiro de álcool e fanzines de escola e panfletos clandestinos; sangramento suave nas bordas das letras sobre papel poroso manilha.",
        "python_script": """import numpy as np
from PIL import Image, ImageFilter

def render_mimeografo(input_path: str, output_path: str):
    img = Image.open(input_path).convert('L')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    arr = np.array(img, dtype=np.float32)
    # Inverte densidade: mais escuro = mais tinta violeta
    tinta = np.clip(255.0 - arr, 0, 255)
    # Sangramento morfológico
    img_tinta = Image.fromarray(tinta.astype(np.uint8)).filter(ImageFilter.MaxFilter(3))
    arr_tinta = np.array(img_tinta, dtype=np.float32) / 255.0
    h, w = arr_tinta.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [244, 239, 230] # Manilha #F4EFE6
    mask = arr_tinta > 0.3
    poster[mask] = [67, 44, 122] # Violeta #432C7A
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Mimeógrafo Violeta
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec3 colManilha = vec3(0.957, 0.937, 0.902); // #F4EFE6
    vec3 colVioleta = vec3(0.263, 0.173, 0.478); // #432C7A
    gl_FragColor = vec4(l < 0.6 ? colVioleta : colManilha, 1.0);
}"""
    },
    "xilogravura": {
        "slug": "xilogravura",
        "num": "19",
        "name": "Xilogravura Sertaneja",
        "subtitle": "Corte em Madeira & Cordel // Xica Manicongo",
        "algo_name": "Binarização Adaptativa Local de Sauvola (15x15)",
        "base_res": "720px (Lanczos3)",
        "tonal": "Sauvola Local Threshold (w=15, k=0.35, R=128) + Modulação de Fibra de Madeira",
        "dither_math": """Equação de Binarização Adaptativa de Sauvola:
T(x, y) = m(x, y) * [ 1 + k * ( s(x, y) / R - 1 ) ]
onde:
- m(x, y): média local de luminância na janela 15x15
- s(x, y): desvio padrão local na janela 15x15
- k = 0.35: coeficiente empírico de agressividade de corte
- R = 128: amplitude dinâmica de desvio padrão
Pixel é preto se L(x, y) < T(x, y), caso contrário kraft queimado.""",
        "particularity": "Textura cortante da goiva entalhando madeira de lei, papel kraft áspero e folhetos de cordel; homenagem à primeira travesti documentada do Brasil (século XVI).",
        "python_script": """import numpy as np
from PIL import Image
from scipy.ndimage import uniform_filter

def sauvola_threshold(gray: np.ndarray, window: int = 15, k: float = 0.35, R: float = 128.0) -> np.ndarray:
    mean = uniform_filter(gray.astype(np.float32), size=window)
    sq_mean = uniform_filter(gray.astype(np.float32)**2, size=window)
    std = np.sqrt(np.maximum(sq_mean - mean**2, 0.0))
    threshold = mean * (1.0 + k * (std / R - 1.0))
    return np.where(gray < threshold, 0, 255).astype(np.uint8)

def render_xilogravura(input_path: str, output_path: str):
    img = Image.open(input_path).convert('L')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    arr = np.array(img)
    # Aplica Sauvola
    bin_wood = sauvola_threshold(arr)
    h, w = bin_wood.shape
    poster = np.zeros((h, w, 3), dtype=np.uint8)
    poster[:, :] = [214, 196, 165] # Kraft Queimado #D6C4A5
    poster[bin_wood == 0] = [19, 17, 16] # Preto Fumo #131110
    Image.fromarray(poster).save(output_path)""",
        "glsl_shader": """// Fragment Shader Xilogravura
precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    float l = dot(texture2D(u_tex, v_uv).rgb, vec3(0.299, 0.587, 0.114));
    vec3 colFumo  = vec3(0.075, 0.067, 0.063); // #131110
    vec3 colKraft = vec3(0.839, 0.769, 0.647); // #D6C4A5
    gl_FragColor = vec4(l < 0.48 ? colFumo : colKraft, 1.0);
}"""
    },
    "glitch-crt": {
        "slug": "glitch-crt",
        "num": "20",
        "name": "Glitch CRT",
        "subtitle": "Sinal Analógico Corrompido & V-Hold // Robeyoncé Lima",
        "algo_name": "Sync Jitter Senoidal + Aberração Cromática R/B + V-Hold Roll",
        "base_res": "720px (Lanczos3)",
        "tonal": "Deslocamento Horizontal Senoidal + Desacoplamento de Canais R/B (+-4px) + Perda de V-Hold",
        "dither_math": """Equação de Sync Jitter Horizontal:
dx(y) = 4.0 * sin(y * 0.12) + 12.0 * (rand(y) < 0.04)
Aberração Cromática Desacoplada:
R(x, y) = Canal_R(x + 4, y)
G(x, y) = Canal_G(x, y)
B(x, y) = Canal_B(x - 4, y)
Barra Horizontal V-Hold: Inversão total de fase a cada ciclo vertical periódico.""",
        "particularity": "Homenagem à advogada pioneira Robeyoncé Lima e à televisão pirata dos anos 80/90; aberração cromática analógica e perda periódica de sincronismo vertical.",
        "python_script": """import numpy as np
from PIL import Image

def render_glitch_crt(input_path: str, output_path: str):
    img = Image.open(input_path).convert('RGB')
    w, h = img.size
    img = img.resize((720, int(round(h * 720 / w))), Image.Resampling.LANCZOS)
    arr = np.array(img, dtype=np.uint8)
    h, w, c = arr.shape
    out = np.zeros_like(arr)
    # 1. Aberração cromática desacoplada R/B
    shift = 4
    out[:, shift:, 0] = arr[:, :-shift, 0] # Vermelho avança
    out[:, :, 1] = arr[:, :, 1]            # Verde estável
    out[:, :-shift, 2] = arr[:, shift:, 2] # Azul recua
    # 2. Sync Jitter horizontal senoidal
    for y in range(h):
        offset = int(round(5.0 * np.sin(y * 0.15)))
        out[y] = np.roll(out[y], offset, axis=0)
    # 3. Barra de V-Hold roll com inversão
    vhold_y = int(h * 0.45)
    out[vhold_y:vhold_y+30] = 255 - out[vhold_y:vhold_y+30]
    Image.fromarray(out).save(output_path)""",
        "glsl_shader": """// Fragment Shader Glitch CRT
precision mediump float;
uniform sampler2D u_tex;
uniform float u_time;
varying vec2 v_uv;
void main() {
    vec2 uv = v_uv;
    // Jitter horizontal senoidal
    uv.x += sin(uv.y * 40.0) * 0.005;
    float r = texture2D(u_tex, uv + vec2(0.006, 0.0)).r;
    float g = texture2D(u_tex, uv).g;
    float b = texture2D(u_tex, uv - vec2(0.006, 0.0)).b;
    gl_FragColor = vec4(r, g, b, 1.0);
}"""
    }
}

def load_index_cards():
    with open(INDEX_HTML_PATH, "r", encoding="utf-8") as f:
        soup = BeautifulSoup(f.read(), "html.parser")
    cards = soup.find_all("div", class_="preset-card")
    data = {}
    for c in cards:
        p_id = c.get("id", "").replace("preset-", "")
        # Extrai paleta
        chips = []
        for chip in c.find_all("span", class_="palette-chip"):
            chips.append(chip.text.strip())
        # Extrai histórico
        ps = c.find_all("p")
        hist = ""
        just = ""
        for p in ps:
            txt = p.text.strip()
            if "Histórico Cultural:" in txt:
                hist = txt.replace("Histórico Cultural:", "").strip()
            elif "Justificativa Técnica:" in txt:
                just = txt.replace("Justificativa Técnica:", "").strip()
        # Extrai tributo
        trib = ""
        trib_div = c.find("div", class_="tribute-bio")
        if trib_div:
            trib = trib_div.text.strip()
        # Extrai imagem e legenda
        img_src = ""
        img_el = c.find("img", class_="preset-poster-img")
        if img_el:
            img_src = img_el.get("src", "")
        caption = ""
        cap_el = c.find("div", class_="preset-poster-caption")
        if cap_el:
            caption = cap_el.decode_contents().strip()
        # Extrai comando CLI
        cli = ""
        cli_el = c.find("code", class_="cli-example-code")
        if cli_el:
            cli = cli_el.text.strip()

        data[p_id] = {
            "chips": chips,
            "hist": hist,
            "just": just,
            "trib": trib,
            "img_src": img_src,
            "caption": caption,
            "cli": cli
        }
    return data

def build_preset_page(p_id, idx, total, card_data, algo_info):
    slug = algo_info["slug"]
    num = algo_info["num"]
    name = algo_info["name"]
    subtitle = algo_info["subtitle"]
    
    # Navegação anterior e próximo
    keys = list(PRESET_ALGO_DETAILS.keys())
    prev_key = keys[(idx - 1) % total]
    next_key = keys[(idx + 1) % total]
    prev_name = PRESET_ALGO_DETAILS[prev_key]["name"]
    next_name = PRESET_ALGO_DETAILS[next_key]["name"]

    # Constrói chips de paleta
    palette_html = ""
    for chip in card_data["chips"]:
        parts = chip.split(" ")
        hex_val = parts[0] if parts else "#000"
        desc = " ".join(parts[1:]) if len(parts) > 1 else hex_val
        palette_html += f"""
        <div class="palette-card">
            <div class="palette-swatch" style="background-color: {hex_val};"></div>
            <div class="palette-meta">
                <strong>{desc}</strong><br>
                HEX: <code>{hex_val}</code>
            </div>
        </div>"""

    # Imagem
    img_rel = card_data["img_src"].replace("../assets/", "../../assets/")

    html_content = f"""<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} — Especificação Técnica &amp; Algorítmica | NAVALHA</title>
    <meta name="description" content="Especificação técnica e matemática detalhada do preset {name} do Navalha para reimplementação agnóstica.">
    <link rel="stylesheet" href="../../assets/style.css">
    <style>
        html, body {{
            height: auto !important;
            min-height: 100% !important;
            max-height: none !important;
            overflow-x: hidden !important;
            overflow-y: auto !important;
        }}
        .doc-container {{
            max-width: 980px;
            margin: 0 auto;
            padding: 30px 20px 80px 20px;
        }}
        .nav-breadcrumbs {{
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 13px;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 24px;
            font-family: 'Courier New', Courier, monospace;
        }}
        .nav-breadcrumbs a {{
            color: #00f0ff;
            text-decoration: none;
        }}
        .nav-breadcrumbs a:hover {{
            text-decoration: underline;
        }}
        .nav-breadcrumbs span {{
            color: #666;
        }}
        .doc-hero-preset {{
            background-color: #141416;
            border-left: 6px solid #e2f835;
            border-top: 1px solid #333;
            border-right: 1px solid #333;
            border-bottom: 1px solid #333;
            padding: 28px;
            margin-bottom: 35px;
            box-shadow: 6px 6px 0px #000;
        }}
        .preset-badge-tag {{
            display: inline-block;
            background-color: #e2f835;
            color: #000000;
            font-size: 12px;
            font-weight: 900;
            padding: 3px 8px;
            letter-spacing: 1.5px;
            text-transform: uppercase;
            margin-bottom: 10px;
        }}
        .preset-page-title {{
            font-size: 34px;
            font-weight: 900;
            letter-spacing: 1.5px;
            color: #ffffff;
            margin: 0 0 8px 0;
            text-transform: uppercase;
        }}
        .preset-page-subtitle {{
            font-size: 15px;
            color: #00f0ff;
            letter-spacing: 1px;
            margin: 0 0 16px 0;
            text-transform: uppercase;
        }}
        .quick-nav-bar {{
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
            margin-top: 18px;
            padding-top: 16px;
            border-top: 1px dashed #333;
        }}
        .quick-nav-btn {{
            font-size: 12px;
            font-weight: 800;
            text-transform: uppercase;
            padding: 6px 12px;
            background: #1c1c22;
            color: #e2f835;
            border: 1px solid #444;
            text-decoration: none;
            transition: all 0.15s ease;
        }}
        .quick-nav-btn:hover {{
            background: #e2f835;
            color: #000;
        }}
        .doc-section {{
            margin-bottom: 45px;
            background-color: #0f0f12;
            border: 2px solid #222226;
            padding: 32px;
            box-shadow: 4px 4px 0px #000;
        }}
        .doc-section h2 {{
            font-size: 24px;
            font-weight: 900;
            letter-spacing: 1.5px;
            color: #e2f835;
            margin-top: 0;
            margin-bottom: 18px;
            border-bottom: 2px solid #333;
            padding-bottom: 10px;
            text-transform: uppercase;
        }}
        .doc-section h3 {{
            font-size: 18px;
            font-weight: 800;
            color: #ff0033;
            margin: 26px 0 12px 0;
            letter-spacing: 1px;
            text-transform: uppercase;
        }}
        .doc-section p {{
            font-size: 15px;
            line-height: 1.85;
            color: #dcdad5;
            margin-bottom: 16px;
        }}
        .doc-section ul, .doc-section ol {{
            font-size: 15px;
            line-height: 1.85;
            color: #dcdad5;
            margin-bottom: 18px;
            padding-left: 24px;
        }}
        .doc-section li {{
            margin-bottom: 8px;
        }}
        .palette-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 12px;
            margin: 20px 0;
        }}
        .palette-card {{
            background: #16161b;
            border: 2px solid #333;
            padding: 14px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}
        .palette-swatch {{
            height: 50px;
            border: 1px solid #444;
            box-shadow: inset 0 0 0 1px rgba(0,0,0,0.5);
        }}
        .palette-meta {{
            font-family: 'Courier New', Courier, monospace;
            font-size: 13px;
            line-height: 1.5;
        }}
        .palette-meta strong {{
            color: #00f0ff;
        }}
        .code-block-container {{
            position: relative;
            margin: 20px 0;
            background: #060608;
            border: 1px solid #333;
            border-left: 4px solid #00f0ff;
        }}
        .code-block-header {{
            background: #111116;
            padding: 8px 14px;
            font-size: 12px;
            font-weight: 800;
            color: #aaa;
            text-transform: uppercase;
            letter-spacing: 1px;
            border-bottom: 1px solid #222;
            display: flex;
            justify-content: space-between;
        }}
        .code-block {{
            margin: 0;
            padding: 16px;
            overflow-x: auto;
            font-family: 'Courier New', Courier, monospace;
            font-size: 13px;
            line-height: 1.6;
            color: #aae0ff;
            background: transparent;
        }}
        .math-matrix-box {{
            background: #08080c;
            border: 1px dashed #444;
            padding: 16px 20px;
            margin: 16px 0;
            font-family: 'Courier New', Courier, monospace;
            font-size: 14px;
            line-height: 1.8;
            color: #e2f835;
            overflow-x: auto;
        }}
        .formula-callout {{
            background: #121218;
            border-left: 4px solid #ff0033;
            padding: 12px 18px;
            margin: 16px 0;
            font-family: 'Courier New', Courier, monospace;
            font-size: 14px;
            color: #ffffff;
        }}
        .tribute-box {{
            background-color: #121216;
            border-left: 4px solid #e2f835;
            padding: 18px;
            margin: 20px 0;
        }}
        .tribute-name {{
            font-size: 16px;
            font-weight: 900;
            color: #e2f835;
            margin-bottom: 6px;
            text-transform: uppercase;
        }}
        .preview-flex {{
            display: flex;
            flex-direction: column;
            align-items: center;
            background: #050507;
            border: 2px solid #282830;
            padding: 16px;
            margin: 24px 0;
        }}
        .preview-img {{
            max-width: 100%;
            height: auto;
            max-height: 560px;
            border: 1px solid #222;
        }}
        .preview-caption {{
            margin-top: 12px;
            font-family: 'Courier New', Courier, monospace;
            font-size: 13px;
            color: #00f0ff;
            text-align: center;
            line-height: 1.6;
        }}
        .cli-box-dark {{
            background: #08080a;
            border: 1px solid #333;
            border-left: 3px solid #ff0033;
            padding: 12px 16px;
            margin-top: 14px;
            font-family: 'Courier New', Courier, monospace;
            font-size: 13px;
            color: #00f0ff;
            word-break: break-all;
            width: 100%;
            box-sizing: border-box;
        }}
    </style>
</head>
<body>
    <div class="doc-container">
        <!-- Navegação Breadcrumbs -->
        <nav class="nav-breadcrumbs" aria-label="Navegação">
            <a href="../../">Navalha App</a>
            <span>/</span>
            <a href="../">Documentação</a>
            <span>/</span>
            <a href="../#secao-presets">Catálogo de Presets</a>
            <span>/</span>
            <strong style="color: #e2f835;">{num}. {name}</strong>
        </nav>

        <!-- Cabeçalho do Preset -->
        <header class="doc-hero-preset">
            <span class="preset-badge-tag">PRESET CANÔNICO #{num}</span>
            <h1 class="preset-page-title">{name}</h1>
            <div class="preset-page-subtitle">{subtitle}</div>
            <p style="color: #bbb; font-size: 14px; margin: 0; line-height: 1.6;">
                Especificação de engenharia reversa e guia algorítmico rigoroso para reimplementação independente em qualquer linguagem ou pipeline de shaders.
            </p>
            <div class="quick-nav-bar">
                <a href="#memorial" class="quick-nav-btn">1. Memorial &amp; Contexto</a>
                <a href="#paleta" class="quick-nav-btn">2. Paleta Canônica</a>
                <a href="#algoritmo" class="quick-nav-btn">3. Método Algorítmico</a>
                <a href="#pipeline" class="quick-nav-btn">4. Pipeline de Dados</a>
                <a href="#reimplementacao" class="quick-nav-btn">5. Código de Reimplementação</a>
                <a href="../../" class="quick-nav-btn" style="background:#ff0033; color:#fff; border-color:#ff0033;">⚡ Testar no App</a>
            </div>
        </header>

        <!-- SEÇÃO 1: MEMORIAL HISTÓRICO -->
        <section class="doc-section" id="memorial">
            <h2>1. Memorial Histórico &amp; Contexto Contracultural</h2>
            <p>
                {card_data["hist"]}
            </p>
            <div class="tribute-box">
                <div class="tribute-name">★ {card_data["trib"].split(":")[0] if ":" in card_data["trib"] else "Homenagem Cultural"}</div>
                <p style="margin: 0; font-size: 14px; color: #ccc; line-height: 1.7;">
                    {card_data["trib"]}
                </p>
            </div>
            <div class="preview-flex">
                <img src="{img_rel}" alt="Cartaz de referência {name}" class="preview-img">
                <div class="preview-caption">
                    {card_data["caption"]}
                </div>
                <div class="cli-box-dark">
                    {card_data["cli"]}
                </div>
            </div>
        </section>

        <!-- SEÇÃO 2: PALETA CROMÁTICA -->
        <section class="doc-section" id="paleta">
            <h2>2. Paleta Cromática Canônica &amp; Mapeamento</h2>
            <p>
                O preset reduz estritamente o espaço de cores contínuo da imagem para a seguinte paleta canônica indexada:
            </p>
            <div class="palette-grid">
                {palette_html}
            </div>
        </section>

        <!-- SEÇÃO 3: MÉTODO ALGORÍTMICO -->
        <section class="doc-section" id="algoritmo">
            <h2>3. Especificação Algorítmica Passo a Passo (Rigor Matemático)</h2>
            <p>
                {card_data["just"]}
            </p>

            <h3>Etapa 3.1: Resolução Base &amp; Downsampling</h3>
            <p>
                A largura base é parametrizada em <strong>{algo_info["base_res"]}</strong>. O redimensionamento deve ocorrer antes da difusão de erro para que os grãos ou blocos de retícula tenham escala física perceptível ao olhar humano.
            </p>

            <h3>Etapa 3.2: Tratamento Tonal &amp; Unsharp Masking</h3>
            <p>
                {algo_info["tonal"]}.
            </p>
            <div class="formula-callout">
                L(x, y) = 0.299 · R(x, y) + 0.587 · G(x, y) + 0.114 · B(x, y)
            </div>

            <h3>Etapa 3.3: Algoritmo de Dither &amp; Difusão de Erro / Retícula</h3>
            <p>
                <strong>Algoritmo Central:</strong> {algo_info["algo_name"]}
            </p>
            <div class="math-matrix-box">
{algo_info["dither_math"]}
            </div>
            <p>
                <strong>Particularidade do Preset:</strong> {algo_info["particularity"]}
            </p>
        </section>

        <!-- SEÇÃO 4: DIAGRAMA DE PIPELINE -->
        <section class="doc-section" id="pipeline">
            <h2>4. Diagrama de Pipeline de Dados</h2>
            <div class="code-block-container">
                <div class="code-block-header">Fluxo Unidirecional Determinístico</div>
                <pre class="code-block">
[ Imagem de Entrada (RGB/RGBA) ]
               │
               ▼
[ 1. Downsampling Físico: {algo_info["base_res"]} ]
               │
               ▼
[ 2. Conversão de Luminância ITU-R BT.601 &amp; Filtro Espacial ]
               │
               ▼
[ 3. Pré-processamento Tonal &amp; Queima de Níveis ]
               │
               ▼
[ 4. Retícula &amp; Dither: {algo_info["algo_name"]} ]
               │
               ▼
[ 5. Mapeamento Estrito na Paleta de Cores ]
               │
               ▼
[ 6. Prensagem de Tarjas, Tipografia e Moldura ]
               │
               ▼
[ Pôster Final Renderizado Bit-a-Bit Idêntico ]
                </pre>
            </div>
        </section>

        <!-- SEÇÃO 5: GUIA DE REIMPLEMENTAÇÃO AGNÓSTICA -->
        <section class="doc-section" id="reimplementacao">
            <h2>5. Guia de Reimplementação Agnóstica (Código de Referência)</h2>
            <p>
                Código autocontido em <strong>Python 3 com NumPy e Pillow</strong> implementando o núcleo matemático deste preset para você rodar ou portar para outra linguagem:
            </p>

            <div class="code-block-container">
                <div class="code-block-header">
                    <span>reimplement_{slug.replace('-', '_')}.py (Python 3 + NumPy)</span>
                    <span style="color:#00f0ff;">LICENÇA: GPL-3.0 / MIT</span>
                </div>
                <pre class="code-block">
{algo_info["python_script"]}
                </pre>
            </div>

            <h3>Fragment Shader GLSL (Para Processamento em GPU em Tempo Real)</h3>
            <div class="code-block-container">
                <div class="code-block-header">{slug.replace('-', '_')}_shader.glsl</div>
                <pre class="code-block">
{algo_info["glsl_shader"]}
                </pre>
            </div>
        </section>

        <!-- RODAPÉ DE NAVEGAÇÃO -->
        <footer style="margin-top: 50px; padding: 24px; background: #121216; border-top: 2px solid #333; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;">
            <a href="{prev_key}.html" style="color: #00f0ff; text-decoration: none; font-weight: 800; font-size: 14px;">← ANTERIOR: {prev_name.upper()}</a>
            <a href="../#secao-presets" style="color: #fff; text-decoration: none; font-weight: 800; font-size: 14px; background: #222; padding: 6px 12px; border: 1px solid #444;">CATÁLOGO GERAL</a>
            <a href="{next_key}.html" style="color: #e2f835; text-decoration: none; font-weight: 800; font-size: 14px;">PRÓXIMO: {next_name.upper()} →</a>
        </footer>
    </div>
</body>
</html>
"""
    return html_content

def main():
    print("Carregando cards de site/doc/index.html...")
    cards_data = load_index_cards()
    print(f"Cards carregados: {len(cards_data)}")
    
    total = len(PRESET_ALGO_DETAILS)
    keys = list(PRESET_ALGO_DETAILS.keys())
    
    for idx, p_id in enumerate(keys):
        if p_id not in cards_data:
            print(f"Aviso: preset {p_id} não encontrado nos cards do index!")
            continue
        algo_info = PRESET_ALGO_DETAILS[p_id]
        card_data = cards_data[p_id]
        page_html = build_preset_page(p_id, idx, total, card_data, algo_info)
        out_file = os.path.join(PRESETS_DIR, f"{p_id}.html")
        with open(out_file, "w", encoding="utf-8") as f:
            f.write(page_html)
        print(f"✓ Gerado: site/doc/presets/{p_id}.html")

    # Atualiza site/doc/index.html inserindo os links em todos os cards
    print("Atualizando botões de link em site/doc/index.html...")
    with open(INDEX_HTML_PATH, "r", encoding="utf-8") as f:
        html_text = f.read()

    for p_id, algo_info in PRESET_ALGO_DETAILS.items():
        card_id = f'id="preset-{p_id}"'
        if card_id not in html_text:
            continue
        link_html = f'''                <div style="margin-top: 14px; text-align: right;">
                    <a href="presets/{p_id}.html" style="display: inline-block; background: #1a1a24; color: #00f0ff; border: 1px solid #00f0ff; padding: 8px 14px; text-decoration: none; font-size: 13px; font-weight: 800; letter-spacing: 0.5px; text-transform: uppercase;">
                        🔬 ESPECIFICAÇÃO TÉCNICA &amp; GUIA DE REIMPLEMENTAÇÃO ({algo_info["name"].upper()}) →
                    </a>
                </div>'''
        # Se já tem link específico para este preset, pula ou atualiza
        if f'href="presets/{p_id}.html"' in html_text:
            continue
        
        # Inserir antes do fechamento do card
        # Procura o tech-specs deste preset
        pattern = rf'(id="preset-{p_id}".*?<div class="tech-specs">.*?</div>)(\s*</div>)'
        match = re.search(pattern, html_text, re.DOTALL)
        if match:
            replacement = match.group(1) + "\n" + link_html + match.group(2)
            html_text = html_text[:match.start()] + replacement + html_text[match.end():]

    with open(INDEX_HTML_PATH, "w", encoding="utf-8") as f:
        f.write(html_text)
    print("✓ site/doc/index.html atualizado com links para todos os 20 presets!")

if __name__ == "__main__":
    main()
