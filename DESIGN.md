# Design System · Kentron Music Hub

## 1. Vision
Precisão matemática unida à vibração acústica. A interface deve ser invisível durante a performance, mas potente durante a preparação. Foco em alto contraste, alvos de toque generosos para músicos em palco e tipografia rigorosa.

## 2. Color Palette
- **Primary (Sovereign Emerald):** `oklch(45% 0.12 160)` - Cor de destaque, botões de ação principal.
- **Surface (Slate Ink):** `oklch(15% 0.02 250)` - Fundo principal, modo escuro profundo para evitar reflexos em palco.
- **Text (Paper White):** `oklch(95% 0.01 250)` - Texto primário, legibilidade máxima.
- **Accent (Vibrant Gold):** `oklch(80% 0.15 85)` - Alertas de regência, marcações de tempo, destaques de acordes.
- **Mute (Stone Grey):** `oklch(60% 0.02 250)` - Texto secundário e bordas sutis.

## 3. Typography
- **Headings:** `IBM Plex Serif` (Bold) - Para títulos de músicas e seções.
- **Body:** `IBM Plex Sans` (Regular/Medium) - Para menus e configurações.
- **Notation/Code:** `IBM Plex Mono` - Para cifras, tablaturas e logs.

## 4. Core Components
- **Chord Card:** Bordas arredondadas (4px), fundo contrastante, tipografia Mono centralizada.
- **Regency Cue:** Banner de largura total, cor de destaque vibrante, texto em caixa alta.
- **Navigation:** Bottom bar com ícones lineares, feedback tátil imediato.

## 5. Interaction Principles
- **No Slop:** Sem sombras excessivas ou gradientes genéricos. Bordas nítidas, transições rápidas.
- **High-Contrast Mode:** Alternância rápida entre "Estúdio" (cinza suave) e "Palco" (preto absoluto/alto contraste).
- **Touch Targets:** Mínimo de 48x48px para qualquer interação em modo de performance.
