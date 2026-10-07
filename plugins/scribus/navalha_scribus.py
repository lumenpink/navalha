#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NAVALHA — Extensão Oficial para Scribus (Scripter API)
======================================================
Automação de editoração aberta para fanzines, tabloides e cartazes no Scribus.
Permite processar quadros de imagem com filtros determinísticos do Navalha e
extrair separações de matrizes diretamente para camadas (layers) do Scribus.

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


def encontrar_binario_navalha():
    """Localiza o binário 'navalha' no PATH ou diretórios padrão."""
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


def executar_navalha(modo, entrada, saida, preset="punk-riot", frase=""):
    """Executa o comando navalha e valida o código de retorno."""
    binario = encontrar_binario_navalha()
    cmd = [binario]

    if modo == "poster":
        cmd.extend(["poster", entrada, "--estetica", preset, "-o", saida])
        if frase and frase.strip():
            cmd.extend(["--frase", frase.strip()])
    elif modo == "dither":
        cmd.extend(["dither", entrada, "-o", saida])
    elif modo == "chapas":
        cmd.extend(["chapas", entrada, "--modo", "cor", "-o", saida])
    else:
        raise ValueError(f"Modo inválido: {modo}")

    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"Erro ao processar imagem no Navalha ({res.returncode}): {res.stderr}")
    return saida


# ==============================================================================
# INTEGRAÇÃO COM SCRIBUS (Scripter API)
# ==============================================================================

def executar_scripter_scribus():
    import scribus

    if not scribus.haveDoc():
        scribus.messageBox("Navalha", "Abra ou crie um documento no Scribus antes de executar o script.", scribus.ICON_WARNING)
        return

    num_sel = scribus.selectionCount()
    if num_sel == 0:
        scribus.messageBox(
            "Navalha",
            "Selecione um Quadro de Imagem (Image Frame) contendo a foto a ser processada.",
            scribus.ICON_WARNING,
        )
        return

    item_selecionado = scribus.getSelectedObject(0)
    tipo_obj = scribus.getObjectType(item_selecionado)

    if tipo_obj != "ImageFrame":
        scribus.messageBox(
            "Navalha",
            f"O objeto selecionado é do tipo '{tipo_obj}'. O Navalha requer um 'ImageFrame' com foto vinculada.",
            scribus.ICON_WARNING,
        )
        return

    arquivo_imagem = scribus.getImageFile(item_selecionado)
    if not arquivo_imagem or not os.path.isfile(arquivo_imagem):
        scribus.messageBox(
            "Navalha",
            "O quadro de imagem selecionado não possui um arquivo de imagem vinculado no disco.",
            scribus.ICON_WARNING,
        )
        return

    # Janela simples de diálogo nativa do Scribus ou Tkinter
    preset_escolhido = "punk-riot"
    frase = "EU SOBREVIVO AO MUNDO"
    modo = "poster"

    try:
        import tkinter as tk
        from tkinter import ttk

        root = tk.Tk()
        root.title("Navalha // Filtro Editorial para Scribus")
        root.geometry("420x300")
        root.resizable(False, False)

        tk.Label(root, text="NAVALHA // MOTOR GRÁFICO PUNK", font=("Helvetica", 12, "bold"), fg="#ff0033").pack(pady=10)

        frame_modo = tk.Frame(root)
        frame_modo.pack(fill="x", padx=20, pady=5)
        tk.Label(frame_modo, text="Modo:").pack(side="left")
        var_modo = tk.StringVar(value="poster")
        combo_modo = ttk.Combobox(frame_modo, textvariable=var_modo, values=["poster", "dither", "chapas"], state="readonly")
        combo_modo.pack(side="right", expand=True, fill="x", padx=5)

        frame_preset = tk.Frame(root)
        frame_preset.pack(fill="x", padx=20, pady=5)
        tk.Label(frame_preset, text="Preset:").pack(side="left")
        var_preset = tk.StringVar(value="punk-riot")
        combo_preset = ttk.Combobox(frame_preset, textvariable=var_preset, values=PRESETS_CANONICOS, state="readonly")
        combo_preset.pack(side="right", expand=True, fill="x", padx=5)

        frame_frase = tk.Frame(root)
        frame_frase.pack(fill="x", padx=20, pady=5)
        tk.Label(frame_frase, text="Frase:").pack(side="left")
        var_frase = tk.StringVar(value="EU SOBREVIVO AO MUNDO")
        entry_frase = tk.Entry(frame_frase, textvariable=var_frase)
        entry_frase.pack(side="right", expand=True, fill="x", padx=5)

        confirmado = [False]

        def on_ok():
            confirmado[0] = True
            root.destroy()

        def on_cancel():
            root.destroy()

        frame_btns = tk.Frame(root)
        frame_btns.pack(pady=20)
        tk.Button(frame_btns, text="Cancelar", command=on_cancel, width=12).pack(side="left", padx=5)
        tk.Button(frame_btns, text="Aplicar Filtro", command=on_ok, width=14, bg="#ff0033", fg="white", font=("Helvetica", 10, "bold")).pack(side="right", padx=5)

        root.mainloop()

        if not confirmado[0]:
            return

        modo = var_modo.get()
        preset_escolhido = var_preset.get()
        frase = var_frase.get()

    except Exception:
        # Fallback para diálogos internos do Scribus se Tkinter não estiver disponível
        resposta_preset = scribus.valueDialog("Navalha Preset", "Digite o preset canônico:", "punk-riot")
        if resposta_preset:
            preset_escolhido = resposta_preset.strip()
        resposta_frase = scribus.valueDialog("Navalha Frase", "Digite a frase da tarja:", "EU SOBREVIVO AO MUNDO")
        if resposta_frase:
            frase = resposta_frase.strip()

    # Processamento e atualização do Scribus
    scribus.statusMessage(f"Navalha: Processando '{preset_escolhido}'...")

    diretorio_base = os.path.dirname(arquivo_imagem)
    nome_base, _ = os.path.splitext(os.path.basename(arquivo_imagem))
    arquivo_saida = os.path.join(diretorio_base, f"{nome_base}_navalha_{preset_escolhido}.png")

    try:
        if modo in ["poster", "dither"]:
            executar_navalha(modo, arquivo_imagem, arquivo_saida, preset=preset_escolhido, frase=frase)
            # Atualiza o quadro de imagem no Scribus com o novo arquivo processado
            scribus.loadImage(arquivo_saida, item_selecionado)
            scribus.statusMessage("Navalha: Imagem atualizada no quadro com sucesso.")
        elif modo == "chapas":
            pasta_chapas = os.path.join(diretorio_base, f"{nome_base}_chapas")
            os.makedirs(pasta_chapas, exist_ok=True)
            executar_navalha("chapas", arquivo_imagem, pasta_chapas, preset=preset_escolhido, frase=frase)

            # Importa as matrizes separadas como camadas dedicadas no Scribus
            arquivos_chapa = sorted([os.path.join(pasta_chapas, f) for f in os.listdir(pasta_chapas) if f.endswith(".png")])
            x, y = scribus.getPosition(item_selecionado)
            w, h = scribus.getSize(item_selecionado)

            for idx, chapa_path in enumerate(arquivos_chapa, start=1):
                nome_camada = f"Navalha_Chapa_{idx}"
                try:
                    scribus.createLayer(nome_camada)
                except Exception:
                    pass
                scribus.setActiveLayer(nome_camada)
                novo_quadro = scribus.createImage(x, y, w, h)
                scribus.loadImage(chapa_path, novo_quadro)
                scribus.setScaleImageToFrame(True, True, novo_quadro)

            scribus.messageBox("Navalha", f"Chapas geradas em {len(arquivos_chapa)} camadas dedicadas!", scribus.ICON_INFORMATION)

    except Exception as e:
        scribus.messageBox("Erro no Navalha", str(e), scribus.ICON_CRITICAL)


# ==============================================================================
# PONTO DE ENTRADA DO SCRIPT
# ==============================================================================

if "scribus" in sys.modules:
    executar_scripter_scribus()
else:
    # Se executado diretamente pelo terminal (testes ou integração)
    if __name__ == "__main__":
        import argparse

        parser = argparse.ArgumentParser(description="Navalha Scribus Bridge (Execução CLI)")
        parser.add_argument("input", help="Arquivo de imagem de entrada")
        parser.add_argument("-o", "--output", required=True, help="Arquivo de saída processado")
        parser.add_argument("-p", "--preset", default="punk-riot", choices=PRESETS_CANONICOS, help="Preset estético")
        parser.add_argument("-m", "--modo", default="poster", choices=["poster", "dither", "chapas"], help="Modo de operação")
        parser.add_argument("-f", "--frase", default="", help="Frase de impacto")

        args = parser.parse_args()
        print(f"[Navalha Scribus Bridge] Processando {args.input} -> {args.output} ({args.preset})...")
        try:
            executar_navalha(args.modo, args.input, args.output, preset=args.preset, frase=args.frase)
            print("[Navalha Scribus Bridge] Concluído com sucesso.")
        except Exception as err:
            print(f"[Navalha Scribus Bridge] Erro: {err}", file=sys.stderr)
            sys.exit(1)
