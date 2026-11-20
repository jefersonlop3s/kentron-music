#!/usr/bin/env bash
# =============================================================================
# Encerrar dispositivos — emulador Android e simulador iOS depois das provas
# =============================================================================
# Regra do projeto (skill `higiene-de-dispositivos`): terminou a prova em
# dispositivo, encerre. Emulador esquecido ocupa RAM, disputa CPU com os gates
# e deixa "Máquinas de pé" aberta no quadro.
#
# Uso:
#     scripts/encerrar-dispositivos.sh             # encerra e verifica
#     scripts/encerrar-dispositivos.sh --dry-run   # só mostra o que faria
#
# Sai 0 quando nada fica rodando; 1 se algo não encerrou ou se a ferramenta
# falhou. Ferramenta ausente é aviso explícito (não há dispositivo daquela
# plataforma para encerrar), nunca fallback silencioso.
#
# Por que não `pgrep -f 'emulator -avd'`: medido em 2026-11-20 — o processo real
# é `qemu/darwin-*` e o padrão não casa. O estado do dispositivo é a fonte.
# =============================================================================
set -uo pipefail

ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ADB="$ANDROID_HOME/platform-tools/adb"
TIMEOUT=20
DRY_RUN=0
[ "${1:-}" = "--dry-run" ] && DRY_RUN=1

falhas=0

lista_android() {
    "$ADB" devices 2>/dev/null \
        | awk 'NR > 1 && $2 ~ /^(device|offline|recovery)$/ { print $1 }'
}

lista_ios() {
    xcrun simctl list devices booted 2>/dev/null \
        | sed -n 's/.*(\([0-9A-Fa-f-]\{36\}\)) (Booted).*/\1/p'
}

# --- Android -----------------------------------------------------------------
if [ -x "$ADB" ]; then
    antes="$(lista_android)"
    if [ -z "$antes" ]; then
        echo "Android: nenhum dispositivo rodando."
    else
        for serial in $antes; do
            case "$serial" in
                emulator-*)
                    if [ "$DRY_RUN" -eq 1 ]; then
                        echo "Android: encerraria $serial"
                        continue
                    fi
                    echo "Android: encerrando $serial…"
                    if ! "$ADB" -s "$serial" emu kill >/dev/null 2>&1; then
                        echo "ERRO: 'adb -s $serial emu kill' falhou" >&2
                        falhas=$((falhas + 1))
                    fi
                    ;;
                *)
                    echo "Android: $serial é dispositivo físico — não encerro."
                    ;;
            esac
        done
        if [ "$DRY_RUN" -eq 0 ]; then
            i=0
            while [ "$i" -lt "$TIMEOUT" ] && [ -n "$(lista_android)" ]; do
                sleep 1
                i=$((i + 1))
            done
            restantes="$(lista_android)"
            if [ -n "$restantes" ]; then
                echo "ERRO: Android ainda lista: $restantes" >&2
                falhas=$((falhas + 1))
            fi
        fi
    fi
else
    echo "Android: adb não encontrado em $ADB — nada a encerrar (instale o SDK para usar emulador)."
fi

# --- iOS ---------------------------------------------------------------------
if command -v xcrun >/dev/null 2>&1; then
    antes="$(lista_ios)"
    if [ -z "$antes" ]; then
        echo "iOS: nenhum simulador bootado."
    elif [ "$DRY_RUN" -eq 1 ]; then
        echo "iOS: encerraria: $(echo "$antes" | tr '\n' ' ')"
    else
        echo "iOS: encerrando simuladores…"
        if ! xcrun simctl shutdown all 2>&1; then
            echo "ERRO: 'xcrun simctl shutdown all' falhou" >&2
            falhas=$((falhas + 1))
        fi
        i=0
        while [ "$i" -lt "$TIMEOUT" ] && [ -n "$(lista_ios)" ]; do
            sleep 1
            i=$((i + 1))
        done
        restantes="$(lista_ios)"
        if [ -n "$restantes" ]; then
            echo "ERRO: iOS ainda lista: $restantes" >&2
            falhas=$((falhas + 1))
        fi
    fi
else
    echo "iOS: xcrun não encontrado — nada a encerrar (sem Xcode nesta máquina)."
fi

if [ "$falhas" -gt 0 ]; then
    echo "FALHOU: $falhas dispositivo(s) não encerraram." >&2
    exit 1
fi
echo "OK: nenhum emulador/simulador rodando."
