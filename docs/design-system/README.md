# Design System · Kentron

Este diretório contém a documentação do sistema visual do Kentron, guiada pela metodologia **Impeccable**.

## 🎨 Filosofia Visual
A interface deve ser invisível durante a performance, mas potente durante a preparação. 
- **Modo Estúdio:** Foco em organização e clareza.
- **Modo Palco:** Foco em alto contraste, legibilidade à distância e alvos de toque generosos.

## 📏 Referências
A fonte de verdade para a UI é o arquivo [DESIGN.md](../../DESIGN.md) na raiz do projeto. 

### Pilares do Design:
1. **Sovereign Emerald:** A cor de destaque que guia a ação.
2. **Slat Ink:** Fundo profundo para evitar reflexos em telas de palco.
3. **IBM Plex:** Tipografia rigorosa para garantir que cifras e notas sejam lidas sem erro.

## 🛠️ Implementação em Flutter
A implementação deve seguir os tokens de design definidos no [DESIGN.md](../../DESIGN.md), utilizando `ThemeData` e componentes customizados que respeitem as regras de acessibilidade (contrastes e touch targets).

