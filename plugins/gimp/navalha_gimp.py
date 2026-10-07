#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NAVALHA — Plugin Oficial para GIMP 2.10 / 3.0
==============================================
Filtro determinístico anti-design e separador de matrizes risográficas/serigráficas
diretamente na camada ativa do GIMP.

Licença: GNU General Public License v3.0 (GPL-3.0-or-later)
Autor: Lumen Pink (Baquara) <hi@lumen.pink>
Repositório: https://codeberg.org/lumenpink/navalha
"""

import os
import sys
import subprocess
import tempfile
import shutil

PRESETS_CANONICOS = [
    "punk-riot",
    "navalha-sata",
    "xerox-podre",
    "viagem-solitaria",
    "lapa-noturna",
    "teatro-operario",
    "risografia",
    "fanzine-digital",
    "asfalto-alerta",
    "vga256",
    "cga",
    "old-mac-pop-art",
    "ascii-art",
    "tabloide-popular",
    "brutalismo",
    "trash-polka",
    "atari-gaming",
    "mimeografo",
    "xilogravura",
    "glitch-crt",
    "punk-80-br",
    "anarco-zine",
    "paulete-furacao",
    "symmy-larrat",
]

MODOS_OPERACAO = ["poster", "dither", "chapas"]


def encontrar_binario_navalha():
    """Localiza o executável 'navalha' no PATH do sistema ou locais habituais."""
    candidatos = [
        shutil.which("navalha"),
        os.path.expanduser("~/.cargo/bin/navalha"),
        "/usr/local/bin/navalha",
        "/usr/bin/navalha",
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/release/navalha")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/debug/navalha")),
    ]
    for c in candidatos:
        if c and os.path.isfile(c) and os.access(c, os.X_OK):
            return c
    return "navalha"


def processar_imagem_navalha(caminho_entrada, caminho_saida, modo="poster", preset="punk-riot", frase=""):
    """Invoca o binário nativo navalha via subprocess."""
    binario = encontrar_binario_navalha()
    cmd = [binario]

    if modo == "poster":
        cmd.extend(["poster", caminho_entrada, "--estetica", preset, "-o", caminho_saida])
        if frase and frase.strip():
            cmd.extend(["--frase", frase.strip()])
    elif modo == "dither":
        cmd.extend(["dither", caminho_entrada, "-o", caminho_saida])
    elif modo == "chapas":
        cmd.extend(["chapas", caminho_entrada, "--modo", "cor", "-o", caminho_saida])
    else:
        raise ValueError(f"Modo desconhecido: {modo}")

    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"Erro ao executar navalha ({res.returncode}): {res.stderr}")
    return caminho_saida


# ==============================================================================
# INTEGRAÇÃO COM GIMP (GimpFu 2.x / Procedural Database)
# ==============================================================================

try:
    from gimpfu import *

    def plugin_navalha(image, drawable, modo_idx, preset_idx, frase, criar_nova_camada):
        """Ponto de entrada chamado pelo GIMP."""
        modo = MODOS_OPERACAO[modo_idx] if 0 <= modo_idx < len(MODOS_OPERACAO) else "poster"
        preset = PRESETS_CANONICOS[preset_idx] if 0 <= preset_idx < len(PRESETS_CANONICOS) else "punk-riot"

        gimp.progress_init(f"Navalha: Processando com preset '{preset}'...")

        with tempfile.TemporaryDirectory() as tmpdir:
            temp_in = os.path.join(tmpdir, "gimp_input.png")
            temp_out = os.path.join(tmpdir, "gimp_output.png")

            # Exporta camada atual como PNG temporário
            pdb.file_png_save_defaults(image, drawable, temp_in, temp_in)
            gimp.progress_update(0.3)

            try:
                processar_imagem_navalha(temp_in, temp_out, modo=modo, preset=preset, frase=frase)
                gimp.progress_update(0.7)

                if os.path.isfile(temp_out):
                    image.undo_group_start()
                    nova_camada = pdb.gimp_file_load_layer(image, temp_out)
                    nova_camada.name = f"Navalha [{preset}]"
                    image.add_layer(nova_camada, 0)

                    if not criar_nova_camada:
                        # Se não for para manter camada anterior, remove a original
                        image.remove_layer(drawable)

                    image.undo_group_end()
                    gimp.displays_flush()
                else:
                    gimp.message("Falha ao gerar o arquivo processado pelo Navalha.")

            except Exception as e:
                gimp.message(f"Erro no filtro Navalha: {str(e)}")

        gimp.progress_update(1.0)

    register(
        "python_fu_navalha_filter",
        "Aplica os filtros determinísticos e anti-design do motor gráfico Navalha.",
        "Integração nativa com o motor gráfico Navalha para pôsteres punk e dither.",
        "Lumen Pink <hi@lumen.pink>",
        "Lumen Pink (GPL-3.0-or-later)",
        "2026",
        "<Image>/Filters/Artistic/Navalha...",
        "RGB*, GRAY*",
        [
            (PF_OPTION, "modo", "Modo de Operação", 0, tuple(m.upper() for m in MODOS_OPERACAO)),
            (PF_OPTION, "preset", "Estética / Preset Canônico", 0, tuple(p.upper() for p in PRESETS_CANONICOS)),
            (PF_STRING, "frase", "Frase de Impacto (opcional)", "EU SOBREVIVO AO MUNDO"),
            (PF_BOOL, "nova_camada", "Preservar camada original (criar nova)", True),
        ],
        [],
        plugin_navalha,
    )

    main()

except ImportError:
    # Se executado fora do GIMP (linha de comando independente ou testes de integração)
    if __name__ == "__main__":
        import argparse

        parser = argparse.ArgumentParser(description="Navalha GIMP Bridge (Execução CLI)")
        parser.add_argument("input", help="Arquivo de imagem de entrada")
        parser.add_argument("-o", "--output", required=True, help="Arquivo de saída processado")
        parser.add_argument("-p", "--preset", default="punk-riot", choices=PRESETS_CANONICOS, help="Preset estético")
        parser.add_argument("-m", "--modo", default="poster", choices=MODOS_OPERACAO, help="Modo de operação")
        parser.add_argument("-f", "--frase", default="", help="Frase de impacto")

        args = parser.parse_args()
        print(f"[Navalha GIMP Bridge] Processando {args.input} -> {args.output} ({args.preset})...")
        try:
            processar_imagem_navalha(args.input, args.output, modo=args.modo, preset=args.preset, frase=args.frase)
            print("[Navalha GIMP Bridge] Concluído com sucesso.")
        except Exception as err:
            print(f"[Navalha GIMP Bridge] Erro: {err}", file=sys.stderr)
            sys.exit(1)
