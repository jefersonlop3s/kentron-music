---
name: alfa-assistido-em-flutter-web
description: O Flutter web renderiza em canvas, entao seletores DOM (getByText, locator) nao funcionam. Para E2E, ou use coordenadas de clique (page.mouse.click) ou use o test runner nativo do Flutter. Playwright com canvas so funciona para screenshots e navegacao por URL.
---

# Alfa assistido em Flutter web — canvas, nao DOM

## O problema

O Flutter web renderiza a interface em um elemento `<canvas>` ou `<flt-glass-pane>`, nao em elementos DOM nativos (div, span, button). Isso significa que:

- `page.getByText('Biblioteca')` nao encontra nada — o texto esta dentro do canvas
- `page.locator('[tooltip="Project"]')` nao funciona — tooltips sao eventos internos do Flutter
- Seletores CSS nao penetram no canvas

## O que funciona

### 1. Coordenadas de clique

Use `page.mouse.click(x, y)` com coordenadas conhecidas do layout. Exemplo do `e2e-web.sh`:

```javascript
await page.mouse.click(640, 525);  // "Abrir cifra"
await page.mouse.click(1218, 28);  // "Copiar link de projecao"
```

**Fragilidade:** qualquer mudanca de layout desloca as coordenadas.

### 2. Navegacao por URL

`page.goto('${BASE}?projecao=${b64}')` funciona porque e uma URL direta, sem depender de interacao com canvas.

### 3. Screenshots

`page.screenshot({ path: '${OUT}/home.png' })` funciona — o canvas renderiza normalmente.

### 4. Test runner nativo do Flutter

`flutter test test/song_reader_page_test.dart` funciona com widgets reais, sem canvas. Use para testar logica de navegacao, estado e renderizacao.

## Recomendacao

| Cenario | Ferramenta |
|---|---|
| Logica de widget (navegacao, estado) | `flutter test` com `pumpWidget` |
| Prova de que o build web compila e serve | `flutter build web` + `curl` |
| Screenshot do app rodando | Playwright com `page.screenshot` |
| Prova de bootstrap por URL | Playwright com `page.goto('?projecao=')` |
| Interacao com botoes/menu | Playwright com coordenadas (fragil) ou teste nativo |