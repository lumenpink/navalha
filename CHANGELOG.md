# Changelog

## [Unreleased]

### Changed
  - (ci) remover campo permissions incompativel com forgejo actions (closes #43)
  - (build) otimizar tamanho de binario com profile.release e habilitar linking estatico musl crt-static (closes #41)

### Features
  - (updater) adequar configuracao opendesktop xdg e verificacao periodica com pin (closes #38)
  - (layout) controle de espacamento vertical entre linhas e palavras em faixas (#28)
  - (core) bordas brancas irregulares de papel rasgado e recortes poligonais (#27)
  - (ui) reestruturar painel de controle em 4 abas funcionais (#25)
  - (ui) lista de frases aleatorias de protesto para texto padrao (#22)
  - (layout) controle de espacamento e prevencao de sobreposicao de palavras (#24)
  - (layout) suporte a formatos e orientacoes horizontal (1100x800) e quadrado (1000x1000) (#23)
  - (core) separacao de chapas para impressao risografia e silk-screen (#3)
  - (core) consolidar motor deterministico navalha com 20 presets canonicos (#1)

### Bug Fixes
  - (ui) prevenir captura acidental de sliders durante a rolagem de tela (#26)
  - (ui) evitar flicker da imagem de perfil padrao durante o carregamento (#21)
  - (ci) garantir resiliencia no deploy e atualizar token cf (#2)
  - (ci) compatibilizar deploy no pages via script nativo para act_runner (#2)
  - (ci) segregar job test para o gate act e job deploy para forgejo (#2)
  - (ci) renomear id do job para test compativel com gate pre-push (#2)

## Previous Releases

