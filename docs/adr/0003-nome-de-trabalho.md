# ADR-0003 · Nome do projeto e do produto: em aberto, com nome de trabalho

- **Status:** em aberto (2026-11-18)
- **Decisão do plano:** D1

## Contexto

A v2.4 propôs quatro duplas (Projeto → App) e afirmou que estavam "verificadas", sem base. Verificação feita em 2026-11-18:

- Contas livres no GitHub: `openmaestro`, `chordmesh`, `kodamesh`, `pulsolivre`, `palcovivo`. Já existem: `koda`, `sincro`, `tacet`.
- **Koda** colide com a KODA, sociedade dinamarquesa de gestão coletiva de música.
- **PalcoVivo** soa só português, o que pesa contra um app multilíngue.
- Não verificados: marcas (INPI, EUIPO, USPTO), domínios, nomes nas lojas.

## Decisão

Seguir com o **nome de trabalho `app-music`** (nome da pasta). Os crates e pacotes do código não carregam marca (`notation_bundle`, `media_bridge`, `core_l10n`…), então o nome definitivo só afeta:

- o identificador do app Flutter (bundle id / application id);
- o nome de exibição e os metadados das lojas;
- o nome da organização e do repositório no GitHub.

Enquanto isso, qualquer identificador de app é placeholder (`org.example.appmusic`).

## Checklist para fechar

1. Escolher 2 a 3 candidatos (a partir das duplas da v2.4 ou novas).
2. Busca de marcas nas classes 9, 41 e 42 no INPI, e no EUIPO/TMview e USPTO.
3. Buscar o nome na App Store, na Google Play, no F-Droid e em domínios `.org` e `.app`.
4. Conferir o significado nos 5 idiomas da faixa 1 e 2.

**Prazo-limite:** antes de criar o app Flutter (bundle id) e antes do primeiro envio às lojas.
