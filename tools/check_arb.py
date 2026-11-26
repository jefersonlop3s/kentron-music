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

# Dicionário de variantes da mesma língua (`tools/variantes.tsv`), que alimenta
# as duas regras — não há lista escondida no Python:
#   1. BLOQUEIA forma da variante no arquivo base (o base fala a variante dele);
#   2. AVISA quando a forma da base não é sobrescrita na variante (radar).
# A regra 1 defende a decisão do dono em 2026-11-21: `pt` é pt-BR, não negociável.
VARIANTES_TSV = Path(__file__).resolve().parent / "variantes.tsv"


def carregar_variantes(caminho: Path) -> list[tuple[str, str, str, str]]:
    """Pares (base, variante, forma-base, forma-variante). Linha torta estoura."""
    if not caminho.is_file():
        return []
    pares: list[tuple[str, str, str, str]] = []
    for numero, linha in enumerate(caminho.read_text(encoding="utf-8").splitlines(), 1):
        texto = linha.strip()
        if not texto or texto.startswith("#"):
            continue
        campos = texto.split("\t")
        if len(campos) != 4 or not all(campos):
            raise ValueError(
                f"{caminho.name}:{numero}: esperado <base> <variante> <forma-base> "
                "<forma-variante>, separados por TAB"
            )
        pares.append((campos[0], campos[1], campos[2], campos[3]))
    return pares


def casar(mensagem: str, palavra: str) -> str | None:
    """Devolve a forma encontrada, tolerando plural simples (`s`/`es`)."""
    padrao = re.compile(rf"\b{re.escape(palavra)}(s|es)?\b", re.IGNORECASE)
    achado = padrao.search(mensagem)
    return achado.group(0) if achado else None


def checar_variantes(
    loaded: dict[str, dict],
    template_keys: set[str],
    pares: list[tuple[str, str, str, str]],
) -> tuple[list[str], list[str]]:
    """Regra 1 (bloqueia) e regra 2 (radar) do dicionário de variantes."""
    erros: list[str] = []
    avisos: list[str] = []
    ja_erro: set[tuple[str, str]] = set()
    ja_aviso: set[tuple[str, str]] = set()
    for base, variante, forma_base, forma_variante in pares:
        dados_base = loaded.get(base)
        if dados_base is None:
            continue
        chaves_variante = set(message_keys(loaded.get(variante, {})))
        for chave in sorted(set(message_keys(dados_base)) & template_keys):
            mensagem = dados_base[chave]
            if not isinstance(mensagem, str):
                continue
            estranha = casar(mensagem, forma_variante)
            if estranha and (base, chave) not in ja_erro:
                # Duas linhas do dicionário podem apontar para a mesma forma da
                # variante (`tela` e `telão` -> `ecrã`): o erro sai uma vez.
                ja_erro.add((base, chave))
                erros.append(
                    f"app_{base}.arb: {chave} usa a forma da variante `{variante}` "
                    f"('{estranha}'); `{base}` fala a variante dele — o texto de "
                    f"`{variante}` vai em app_{variante}.arb"
                )
            tem_forma_base = casar(mensagem, forma_base)
            if (
                tem_forma_base
                and chave not in chaves_variante
                and (base, chave) not in ja_aviso
            ):
                ja_aviso.add((base, chave))
                avisos.append(
                    f"app_{base}.arb: {chave} tem '{tem_forma_base}' e app_{variante}.arb "
                    f"não sobrescreve essa chave (esperado conter '{forma_variante}')"
                )
    return erros, avisos


# `{name}` simples; ignora ramos de plural/select como `=0{Nenhuma}` ou `other{...}`.
SIMPLE_PLACEHOLDER = re.compile(r"([=\w]*)\{(\w+)\}")


def message_keys(data: dict) -> list[str]:
    return [k for k in data if not k.startswith("@")]


def used_placeholders(message: str) -> set[str]:
    return {
        m.group(2) for m in SIMPLE_PLACEHOLDER.finditer(message) if m.group(1) == ""
    }


def declared_placeholders(template: dict, key: str) -> set[str]:
    meta = template.get(f"@{key}", {})
    return set(meta.get("placeholders", {}))


def check(
    directory: Path, variantes: Path | None = None
) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    files = sorted(directory.glob("app_*.arb"))
    if not files:
        return [f"nenhum app_*.arb em {directory}"], []

    loaded: dict[str, dict] = {}
    for path in files:
        locale = path.stem.removeprefix("app_")
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"{path.name}: JSON inválido ({exc})")
            continue
        if data.get("@@locale") != locale:
            errors.append(
                f"{path.name}: @@locale={data.get('@@locale')!r}, esperado {locale!r}"
            )
        loaded[locale] = data

    template = loaded.get(TEMPLATE)
    if template is None:
        return errors + [f"template app_{TEMPLATE}.arb ausente ou inválido"], []
    template_keys = set(message_keys(template))

    for locale, data in loaded.items():
        keys = set(message_keys(data))
        unknown = keys - template_keys
        if unknown:
            errors.append(
                f"app_{locale}.arb: chaves que não existem no template: {sorted(unknown)}"
            )
        if locale in TIER1:
            missing = template_keys - keys
            if missing:
                errors.append(
                    f"app_{locale}.arb: faltam chaves da faixa 1: {sorted(missing)}"
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
                    errors.append(
                        f"app_{locale}.arb: {key} sem placeholder(s) {sorted(missing_ph)}"
                    )
            unexpected = found - expected
            if unexpected:
                errors.append(
                    f"app_{locale}.arb: {key} usa placeholder(s) não declarado(s) {sorted(unexpected)}"
                )

    erros_var, avisos = checar_variantes(
        loaded,
        template_keys,
        carregar_variantes(VARIANTES_TSV if variantes is None else variantes),
    )
    return errors + erros_var, avisos


def main() -> int:
    directory = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_DIR
    try:
        errors, avisos = check(directory)
    except ValueError as exc:
        print(f"ERRO: {exc}")
        return 1
    for error in errors:
        print(f"ERRO: {error}")
    if errors:
        return 1
    count = len(list(directory.glob("app_*.arb")))
    print(f"ok: {count} arquivos ARB consistentes em {directory}")
    for aviso in avisos:
        print(f"AVISO: {aviso}")
    if avisos:
        print(
            f"{len(avisos)} aviso(s) de variante — não reprova; é a chave que a variante ainda não cobre."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
