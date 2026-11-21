---
name: hinario-e-dominio-publico
description: Como decidir se um hino pode ter letra no app e como registrar a ficha dele. Fixa a identidade pelo par (hinário, número), os três vereditos mais o estado "sem ficha", a conta pela morte mais recente entre autoria, tradução e melodia, e a procedência por campo. Carregar ao ingerir ficha de hinário, ao classificar hino do repertório e ao avaliar a camada de tradução brasileira.
---

# Ficha de hino e domínio público

Caso de origem (2026-11-21): repertório real de 10 hinos do dono, fichas lidas do **Cantor Cristão
Cifrado** (EME/JUERP) e, para o único hino de outro hinário, da **Harpa Cristã** em PDF. Três erros
custaram a sessão e viraram regra:

- **Inferi a ficha pelo número** ("Exultação | 015" = hino 01) — é o **hino 15**; o 01 é *Antífona*.
- A fonte web (`revistaebd.com.br/cantor-cristao/<n>`) **omite o tradutor em todos os hinos
  cruzados**: no *112 Vencendo vem Jesus* ela lista Steffe e Howe (1819-1910) e cala o **Pitrowsky
  (1965)** — quem se guiasse por ela concluiria **domínio público**, o oposto do correto.
- **Cantor Cristão 291 é "Comunhão Celeste"** e **Harpa Cristã 291 é "A Mensagem da Cruz"**: o número
  sozinho não identifica nada.

Placar medido dos 10: **6 domínio público** (09, 15, 46, 135, 295, 377), **3 protegidos**
(385→2033, 112→2036, 291→2040) e **1 GAP** (92). **Quatro dos dez exigem "ficha + letra do usuário"**.

## Regras

1. **Identidade é o par (hinário, número) + título.** O número só vale dentro do hinário.
2. **Três vereditos e um estado:** `pd` (com o ano), `protegido` (com o ano), `gap` (falta o dado que
   permite afirmar) e `sem_ficha`. **Nunca** "provavelmente livre".
3. **A conta é pela morte mais recente entre autoria, tradução e melodia.** No Brasil: 70 anos
   contados de 1º de janeiro do ano seguinte ao falecimento — na prática, `ano_da_morte + 71`.
4. **A camada que costuma mandar é a tradução brasileira do século XX**, não a letra original.
   Casos reais: 291 (A.A., 1969 → 2040); 112 (Pitrowsky, 1965 → 2036); 385 (Avelino de Souza, 1962 → 2033).
5. **Fonte primária é o exemplar impresso.** O *Cantor Cristão Cifrado* imprime `Autoria:`,
   `Tradução:`, `Melodia:` **com anos** em cada hino. Fonte web é secundária: além de omitir o
   tradutor, errou a data do MacKay no 135 (deu 1847-1929, que são as datas do **tradutor** daquele
   hino no impresso).
6. **Procedência por campo** — quem trouxe autoria, quem trouxe tradução, quem trouxe melodia. O
   crédito pode vir do próprio hino (sigla) e o nome/datas de índice e biografia.
7. **A edição cifrada é obra protegida** (Cantor Cristão Cifrado, EME/JUERP 2011; Harpa Cristã
   Cifrada, CPAD © 2007 — "todos os direitos reservados para a língua portuguesa"): entram **só
   fatos**. Letra e cifra da edição **nunca**; fotos e PDFs **fora do repositório**.
8. **GAP é resposta, não falha.** Tradutor sem data de falecimento no livro (92, *D. M. Hazlett
   1852-?*) ⇒ **não se afirma** queda de domínio.
9. **Consequência no app:** hino protegido ou gap ⇒ **ficha + letra/cifra do próprio usuário**;
   domínio público ⇒ letra livre. A cifra continua vindo do áudio (o outro eixo do ADR-0012).

## O que NÃO fazer

- **Não preencher lacuna por inferência** — o número não substitui a leitura da fonte.
- **Não usar fonte web como primária** para decidir veredito.
- **Não tratar título como identidade** — há **dois hinos chamados "Louvor"** no mesmo repertório
  (135, domínio público; 385, protegido até 2033), com vereditos opostos.
- **Não confundir as camadas**: o hino pode ser livre e a **tradução** que se canta não ser.
