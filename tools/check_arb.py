#!/usr/bin/env python3
"""Portão de i18n que não depende do Flutter (plano v2.5, R0).

Regras:
  * todo `app_*.arb` é JSON válido e o `@@locale` bate com o nome do arquivo;
  * idiomas da faixa 1 (en, pt, es) têm TODAS as chaves do template (app_en.arb);
  * variantes regionais (ex.: pt_PT, es_ES) só podem sobrescrever chaves existentes;
  * toda mensagem usa exatamente os placeholders declarados no template;
  * `app_pt.arb` é **português do Brasil** (ADR-0002, faixa 1): grafia de português
    europeu reprova o gate — a variante europeia vive em `app_pt_PT.arb`.

Uso: python3 tools/check_arb.py [diretório-com-os-arb]
"""

import json
import re
import sys
from pathlib import Path

TIER1 = {"en", "pt", "es"}
TEMPLATE = "en"
DEFAULT_DIR = Path(__file__).resolve().parent.parent / "packages/core_l10n/lib/l10n"

# `pt` é português do Brasil (ADR-0002, faixa 1) — decisão do dono em
# 2026-11-21, "não negociável". Marcas de português europeu não entram neste
# arquivo; quem quiser a variante europeia sobrescreve em `app_pt_PT.arb`.
# É gate, não prosa: sem ele, "ecrã" passa e o app fala português de Portugal
# com o usuário brasileiro.
PT_BR = "pt"
MARCADORES_PT_PT = (
    "ecrã",
    "secção",
    "utilizador",
    "telemóvel",
    "autocarro",
    "connosco",
    "receção",
    "ficheiro",
)
MARCADOR_PT_PT = re.compile(
    r"\b(" + "|".join(MARCADORES_PT_PT) + r")(s|es)?\b",
    re.IGNORECASE,
)

# `{name}` simples; ignora ramos de plural/select como `=0{Nenhuma}` ou `other{...}`.
SIMPLE_PLACEHOLDER = re.compile(r"([=\w]*)\{(\w+)\}")


def message_keys(data: dict) -> list[str]:
    return [k for k in data if not k.startswith("@")]


def used_placeholders(message: str) -> set[str]:
    return {m.group(2) for m in SIMPLE_PLACEHOLDER.finditer(message) if m.group(1) == ""}


def declared_placeholders(template: dict, key: str) -> set[str]:
    meta = template.get(f"@{key}", {})
    return set(meta.get("placeholders", {}))


def check(directory: Path) -> list[str]:
    errors: list[str] = []
    files = sorted(directory.glob("app_*.arb"))
    if not files:
        return [f"nenhum app_*.arb em {directory}"]

    loaded: dict[str, dict] = {}
    for path in files:
        locale = path.stem.removeprefix("app_")
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"{path.name}: JSON inválido ({exc})")
            continue
        if data.get("@@locale") != locale:
            errors.append(f"{path.name}: @@locale={data.get('@@locale')!r}, esperado {locale!r}")
        loaded[locale] = data

    template = loaded.get(TEMPLATE)
    if template is None:
        return errors + [f"template app_{TEMPLATE}.arb ausente ou inválido"]
    template_keys = set(message_keys(template))

    for locale, data in loaded.items():
        keys = set(message_keys(data))
        unknown = keys - template_keys
        if unknown:
            errors.append(f"app_{locale}.arb: chaves que não existem no template: {sorted(unknown)}")
        if locale in TIER1:
            missing = template_keys - keys
            if missing:
                errors.append(f"app_{locale}.arb: faltam chaves da faixa 1: {sorted(missing)}")
        if locale == PT_BR:
            for key in sorted(keys & template_keys):
                mensagem = data[key]
                if not isinstance(mensagem, str):
                    continue
                achado = MARCADOR_PT_PT.search(mensagem)
                if achado:
                    errors.append(
                        f"app_pt.arb: {key} usa grafia de português europeu "
                        f"('{achado.group(0)}'); `pt` é pt-BR (ADR-0002) — "
                        "a variante europeia vai em pt_PT"
                    )
        for key in keys & template_keys:
            message = data[key]
            if not isinstance(message, str):
                errors.append(f"app_{locale}.arb: {key} não é string")
                continue
            expected = declared_placeholders(template, key)
            found = used_placeholders(message)
            if expected:
                missing_ph = {p for p in expected if f"{{{p}" not in message}
                if missing_ph:
                    errors.append(f"app_{locale}.arb: {key} sem placeholder(s) {sorted(missing_ph)}")
            unexpected = found - expected
            if unexpected:
                errors.append(f"app_{locale}.arb: {key} usa placeholder(s) não declarado(s) {sorted(unexpected)}")
    return errors


def main() -> int:
    directory = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_DIR
    errors = check(directory)
    for error in errors:
        print(f"ERRO: {error}")
    if errors:
        return 1
    count = len(list(directory.glob("app_*.arb")))
    print(f"ok: {count} arquivos ARB consistentes em {directory}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
