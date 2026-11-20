# core_l10n

Recursos de tradução compartilhados por todos os pacotes do app (plano v2.5, seção de
internacionalização).

## Estrutura

- `lib/l10n/app_en.arb`: template e idioma-fonte (`en-US`, decisão D8).
- `lib/l10n/app_pt.arb`: base de `pt-BR`. `app_pt_PT.arb` só sobrescreve o que difere.
- `lib/l10n/app_es.arb`: base de `es-419`. `app_es_ES.arb` só sobrescreve o que difere.
- Faixa 1 (en, pt, es) precisa estar 100% traduzida; regionais herdam da base.

## Regras

1. Chaves de aviso de palco (`cue*`) são curtas: aparecem em telas de regência.
2. O Rust nunca devolve texto para o usuário: devolve chaves (ex.: `chord.parse.empty_bass`)
   que a camada Flutter mapeia para mensagens daqui.
3. Mudou texto em inglês? Atualize pt e es na mesma PR; o CI reprova se faltar chave ou
   se placeholders divergirem (`python3 tools/check_arb.py`).

## Estado

- **As traduções são rascunho** e precisam de revisão de falante nativo de cada variante
  antes do R1. Células ainda sem tradução regional (ex.: `roleWorshipLeader` em pt-PT e
  es-ES) herdam da base de propósito: estão marcadas "a validar" no plano.
- **Não validado com o Flutter:** o ambiente de criação não tinha Flutter/Dart. Falta rodar
  `flutter gen-l10n`, confirmar as versões em `pubspec.yaml` e criar `lib/core_l10n.dart`
  (`flutter create --template=package`). O `tools/check_arb.py` cobre a paridade de chaves
  e placeholders sem depender do Flutter.
