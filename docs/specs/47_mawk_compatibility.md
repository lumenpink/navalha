# Especificação Formal: Compatibilização do Script de Teste CI com mawk (#47)

## 1. Contexto e Motivação
O script `tests/test_ci_decouple_jobs.sh` foi introduzido para auditar a conformidade estrutural do workflow `.forgejo/workflows/ci.yml`. No entanto, na imagem de contêiner do runner (`gost-ci-cross:latest`), o utilitário `awk` padrão do Debian é o `mawk 1.3.4`. 

Diferentemente do GNU `gawk`, o `mawk` clássico não suporta expressões regulares com quantificadores de intervalo no formato `{2}` (ex.: `^[[:space:]]{2}`), fazendo com que as seções dos jobs não fossem extraídas e gerando falsos negativos no CI.

## 2. Requisitos da Especificação
- A extração de seções e linhas de indentação de 2 espaços em `tests/test_ci_decouple_jobs.sh` deve utilizar correspondência direta e universal (`^  `), compatível com qualquer variante de `awk` (`mawk`, `gawk`, `nawk`, `busybox awk`).
- Os testes estruturais e a suíte de 4 mutantes devem ser executados com sucesso tanto no ambiente host quanto no contêiner com `mawk`.
