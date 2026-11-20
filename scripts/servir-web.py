#!/usr/bin/env python3
"""Serve o alvo web do Kentron com os cabecalhos que a ponte exige (ADR-0007).

O wasm do nucleo Rust e compilado pelo `flutter_rust_bridge_codegen build-web`
com shared memory (`-Z build-std` + atomics), entao o navegador so o instancia
se a pagina vier com:

    Cross-Origin-Opener-Policy: same-origin
    Cross-Origin-Embedder-Policy: require-corp

Sem esses cabecalhos o `SharedArrayBuffer` nao existe e a ponte falha ao
carregar. O `flutter build web`/`flutter run -d web-server` do Flutter 3.50.1
nao enviam esses cabecalhos, dai este servidor minimo.

Uso (da raiz do repositorio, depois de `flutter build web --release`):

    python3 scripts/servir-web.py
    python3 scripts/servir-web.py --porta 9000 --dir apps/kentron_app/build/web
"""

from __future__ import annotations

import argparse
import functools
import http.server
import socketserver
import sys
from pathlib import Path

CABECALHOS_DE_ISOLAMENTO = {
    "Cross-Origin-Opener-Policy": "same-origin",
    "Cross-Origin-Embedder-Policy": "require-corp",
    "Cache-Control": "no-store",
}

TIPOS = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".mjs": "text/javascript; charset=utf-8",
    ".json": "application/json; charset=utf-8",
    ".wasm": "application/wasm",
    ".css": "text/css; charset=utf-8",
    ".png": "image/png",
    ".svg": "image/svg+xml",
    ".ico": "image/x-icon",
    ".ttf": "font/ttf",
    ".otf": "font/otf",
    ".woff2": "font/woff2",
    ".map": "application/json",
}


class Servidor(socketserver.ThreadingTCPServer):
    """TCPServer que aceita reuso de endereco (evita TIME_WAIT entre execucoes)."""

    allow_reuse_address = True


class Handler(http.server.SimpleHTTPRequestHandler):
    """Servidor estatico que acrescenta os cabecalhos de isolamento."""

    def guess_type(self, path):  # noqa: D102 - assinatura da stdlib
        sufixo = Path(path).suffix
        if sufixo in TIPOS:
            return TIPOS[sufixo]
        return super().guess_type(path)

    def end_headers(self):  # noqa: D102 - assinatura da stdlib
        for nome, valor in CABECALHOS_DE_ISOLAMENTO.items():
            self.send_header(nome, valor)
        super().end_headers()

    def log_message(self, fmt, *args):  # noqa: D102 - assinatura da stdlib
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))


def main() -> int:
    raiz = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dir",
        default=str(raiz / "apps" / "kentron_app" / "build" / "web"),
        help="diretorio do build web (default: apps/kentron_app/build/web)",
    )
    parser.add_argument("--porta", type=int, default=8080, help="porta (default: 8080)")
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help=(
            "interface (default: 127.0.0.1, so a maquina local). Use 0.0.0.0 "
            "apenas quando outro host precisar alcancar o servidor - isso "
            "expoe o dev server na rede em que a maquina estiver."
        ),
    )
    args = parser.parse_args()

    diretorio = Path(args.dir).resolve()
    # Fail-loud: sem o build, avisar em vez de servir 404 em tudo.
    if not (diretorio / "index.html").is_file():
        print(
            f"sem build web em {diretorio}\n"
            "construa antes:\n"
            "  cd apps/kentron_app && flutter build web --release",
            file=sys.stderr,
        )
        return 1
    if not (diretorio / "pkg" / "kentron_ffi.js").is_file():
        print(
            f"o build em {diretorio} nao tem pkg/kentron_ffi.js\n"
            "regenere a ponte antes de buildar:\n"
            "  cd apps/kentron_app && flutter_rust_bridge_codegen build-web \\\n"
            "    --dart-root . --rust-root ../../crates/kentron_ffi --release",
            file=sys.stderr,
        )
        return 1

    handler = functools.partial(Handler, directory=str(diretorio))
    with Servidor((args.host, args.porta), handler) as servidor:
        print(f"servindo {diretorio}")
        print(f"  em http://{args.host}:{args.porta}/ (Ctrl+C para parar)")
        if args.host not in ("127.0.0.1", "localhost"):
            print("  atencao: escutando alem do local - o dev server esta exposto")
        print("  cabecalhos COOP/COEP ativos - a ponte wasm carrega")
        try:
            servidor.serve_forever()
        except KeyboardInterrupt:
            print("\nparado")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
