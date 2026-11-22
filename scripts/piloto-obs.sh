#!/usr/bin/env bash
# Piloto automatizado do OBS (ADR-0010): prova o Browser Source de verdade.
#
# O que prova: o app servido na web, aberto dentro do Browser Source do OBS,
# renderiza a projeção; trocar de seção muda o que o OBS mostra; e o ciclo se
# repete em duas rodadas, como manda a ADR-0010.
#
# O que NÃO prova (e onde está a prova):
# - transposição de tom: a projeção não mostra acorde, então transpor não muda
#   o pixel. A transposição é provada em `scripts/e2e-web.sh` e nos testes Rust.
# - o app em si: idem, `scripts/e2e-web.sh`.
#
# Uso:
#   scripts/piloto-obs.sh               # duas rodadas
#   scripts/piloto-obs.sh --negativo    # prova negativa: URL morta tem de reprovar
#   scripts/piloto-obs.sh --construir   # constrói o build web antes
#
# Pré-requisitos: SSH sem senha para a VM, obs-websocket ligado nela e Node 22+
# (WebSocket nativo — sem dependência nova). O túnel SSH leva a porta 4455, então
# o firewall da VM não precisa ser aberto.
#
# Por que o OBS abre a página em `http://127.0.0.1` e não pelo IP da LAN: a ponte
# wasm do núcleo Rust é compilada com memória compartilhada e só instancia com
# `SharedArrayBuffer`, que o navegador libera apenas em contexto seguro.
# `localhost` conta como seguro; `http://192.168.x.x` não. Servido pelo IP, o CEF
# do OBS ignora COOP/COEP, cai no caminho sem worker pool e morre com
# `Identifier 'wasm_bindgen' has already been declared` — a fonte fica em branco.
# O túnel reverso (`ssh -R`) põe o servidor do Mac em `127.0.0.1` dentro da VM.
set -euo pipefail

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VM="${KENTRON_VM:-windows11}"
PORTA="${KENTRON_PORTA:-8765}"
NEGATIVO=0
CONSTRUIR=0
SHOTS="/tmp/kentron-piloto/shots"
DRIVER="/tmp/kentron-piloto/driver.mjs"

for arg in "$@"; do
  case "$arg" in
    --negativo) NEGATIVO=1 ;;
    --construir) CONSTRUIR=1 ;;
    *) echo "argumento desconhecido: $arg" >&2; exit 2 ;;
  esac
done

passo() { printf '\n==> %s\n' "$*"; }

# ---------------------------------------------------------------- servidor ---

construir_web() {
  passo "Build web (o wasm da ponte vai em web/pkg, depois o flutter copia)"
  ( cd "$RAIZ/apps/kentron_app" \
      && flutter_rust_bridge_codegen build-web --dart-root . \
           --rust-root ../../crates/kentron_ffi --release >/dev/null \
      && flutter build web --release >/dev/null )
}

if [ "$CONSTRUIR" = 1 ]; then
  construir_web
fi

if [ ! -f "$RAIZ/apps/kentron_app/build/web/pkg/kentron_ffi.js" ]; then
  echo "sem build web com a ponte: rode com --construir" >&2
  exit 1
fi

# O OBS abre a página em `127.0.0.1:$PORTA` dentro da VM: o túnel reverso põe o
# servidor do Mac lá. `localhost` é contexto seguro, então COOP/COEP valem e o
# `SharedArrayBuffer` existe — sem isso a ponte wasm não carrega (ver cabeçalho).
BASE="http://127.0.0.1:$PORTA"

passo "Servindo o build em 127.0.0.1:$PORTA (túnel reverso até a VM)"
mkdir -p /tmp/kentron-piloto
python3 "$RAIZ/scripts/servir-web.py" --porta "$PORTA" \
  >/tmp/kentron-piloto/servidor.log 2>&1 &
SERVIDOR=$!
TUNEL=""
limpar() {
  [ -n "$TUNEL" ] && kill "$TUNEL" 2>/dev/null || true
  kill "$SERVIDOR" 2>/dev/null || true
}
trap limpar EXIT

for _ in $(seq 1 40); do
  curl -sf -o /dev/null "$BASE/" && break
  sleep 0.25
done
curl -sf -o /dev/null "$BASE/pkg/kentron_ffi.js" \
  || { echo "o servidor não entregou pkg/kentron_ffi.js" >&2; exit 1; }

# ------------------------------------------------------------- obs-websocket ---

passo "Túneis SSH: 4455 do obs-websocket e $PORTA de volta para o servidor"
ssh -N -L 4455:127.0.0.1:4455 -R "$PORTA:127.0.0.1:$PORTA" \
  -o BatchMode=yes -o ExitOnForwardFailure=yes "$VM" &
TUNEL=$!

passo "Conferindo que a VM alcança o servidor por 127.0.0.1:$PORTA"
CODIGO="$(ssh -o BatchMode=yes "$VM" \
  "powershell -NoProfile -Command \"(Invoke-WebRequest -UseBasicParsing '$BASE/' -TimeoutSec 8).StatusCode\"" \
  2>/dev/null | tr -d '\r\n ' || true)"
if [ "$CODIGO" != "200" ]; then
  echo "a VM não alcançou $BASE (status '$CODIGO')" >&2
  echo "o túnel reverso não subiu: confira AllowTcpForwarding no sshd da VM" >&2
  exit 1
fi
for _ in $(seq 1 20); do
  nc -z -G 1 127.0.0.1 4455 2>/dev/null && break
  sleep 0.25
done

# A senha é lida da fonte (o config do OBS na VM) e vai por variável de ambiente,
# sem aparecer em log nem em argumento de linha de comando.
OBS_SENHA="$(ssh -o BatchMode=yes "$VM" \
  "powershell -NoProfile -Command \"(Get-Content \\\"\$env:APPDATA\\obs-studio\\plugin_config\\obs-websocket\\config.json\\\" -Raw | ConvertFrom-Json).server_password\"" \
  2>/dev/null | tr -d '\r\n')"
if [ -z "$OBS_SENHA" ]; then
  echo "obs-websocket sem senha configurada na VM" >&2
  exit 1
fi

# ------------------------------------------------------------------- driver ---

mkdir -p "$SHOTS"
rm -f "$SHOTS"/*.png

cat > "$DRIVER" <<'JS'
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';
import { inflateSync } from 'node:zlib';

const OBS = process.env.OBS_URL;
const SENHA = process.env.OBS_SENHA;
const BASE = process.env.KENTRON_BASE;
const SHOTS = process.env.KENTRON_SHOTS;
const NEGATIVO = process.env.KENTRON_NEGATIVO === '1';

// Nome da fonte criada no OBS. Fica no escopo do módulo porque a captura
// acontece fora de `main`.
const FONTE = 'Kentron Piloto';

// Cifra com duas seções: a troca de seção é o que muda o pixel na projeção.
const CIFRA = [
  '{title: Piloto}',
  '{key: C}',
  '[Verse]',
  '[C]Santo, santo, santo',
  '[Chorus]',
  '[G]Toda a terra canta',
  '',
].join('\n');

const b64 = (texto) => Buffer.from(texto, 'utf8').toString('base64');

// URL da projeção numa seção. `t` é cache-buster: sem ele o OBS pode reusar a
// página anterior e a captura não muda.
const urlDa = (secao, marca) =>
  `${BASE}/?projecao=${b64(CIFRA)}&secao=${secao}&t=${marca}`;

// ---------------------------------------------------------------- protocolo ---

let ws;
let proximoId = 1;
const pendentes = new Map();

function conectar() {
  return new Promise((resolver, rejeitar) => {
    ws = new WebSocket(OBS);
    ws.addEventListener('message', (evento) => {
      const msg = JSON.parse(evento.data);
      if (msg.op === 0) {
        const desafio = msg.d?.authentication;
        const dados = { rpcVersion: 1 };
        if (desafio) {
          const { challenge, salt } = desafio;
          const segredo = createHash('sha256')
            .update(SENHA + salt)
            .digest('base64');
          dados.authentication = createHash('sha256')
            .update(segredo + challenge)
            .digest('base64');
        }
        ws.send(JSON.stringify({ op: 1, d: dados }));
      } else if (msg.op === 2) {
        resolver();
      } else if (msg.op === 7) {
        const { requestId, requestStatus, responseData } = msg.d;
        const pendente = pendentes.get(requestId);
        if (!pendente) return;
        pendentes.delete(requestId);
        if (requestStatus.result) pendente.resolver(responseData ?? {});
        else pendente.rejeitar(new Error(requestStatus.comment ?? 'falhou'));
      }
    });
    ws.addEventListener('error', () =>
      rejeitar(new Error(`não conectou em ${OBS}`)),
    );
  });
}

function pedir(requestType, requestData = {}) {
  const requestId = `r${proximoId++}`;
  return new Promise((resolver, rejeitar) => {
    pendentes.set(requestId, { resolver, rejeitar });
    ws.send(
      JSON.stringify({ op: 6, d: { requestType, requestId, requestData } }),
    );
  });
}

// --------------------------------------------------------------------- PNG ---

// Decodifica PNG 8 bits (RGB/RGBA/cinza) sem dependência: parse dos chunks,
// inflate do IDAT e desfiltro das linhas. Devolve as cores RGB distintas e o
// desvio-padrão da luminância — é o que separa "página em branco" de "projeção".
function lerPng(base64) {
  const dados = Buffer.from(base64, 'base64');
  let largura = 0;
  let altura = 0;
  let tipo = 0;
  const idat = [];
  let pos = 8;
  while (pos < dados.length) {
    const tamanho = dados.readUInt32BE(pos);
    const nome = dados.toString('ascii', pos + 4, pos + 8);
    const corpo = dados.subarray(pos + 8, pos + 8 + tamanho);
    if (nome === 'IHDR') {
      largura = corpo.readUInt32BE(0);
      altura = corpo.readUInt32BE(4);
      tipo = corpo[9];
    } else if (nome === 'IDAT') {
      idat.push(corpo);
    } else if (nome === 'IEND') {
      break;
    }
    pos += 12 + tamanho;
  }
  const canais = { 0: 1, 2: 3, 4: 2, 6: 4 }[tipo];
  if (!canais) throw new Error(`PNG com tipo de cor não suportado: ${tipo}`);
  const bruto = inflateSync(Buffer.concat(idat));
  const passo = largura * canais;
  const saida = Buffer.alloc(passo * altura);
  let anterior = Buffer.alloc(passo);
  for (let linha = 0; linha < altura; linha++) {
    const filtro = bruto[linha * (passo + 1)];
    const atual = Buffer.from(
      bruto.subarray(linha * (passo + 1) + 1, linha * (passo + 1) + 1 + passo),
    );
    for (let i = 0; i < passo; i++) {
      const a = i >= canais ? atual[i - canais] : 0;
      const b = anterior[i];
      const c = i >= canais ? anterior[i - canais] : 0;
      let valor = atual[i];
      if (filtro === 1) valor += a;
      else if (filtro === 2) valor += b;
      else if (filtro === 3) valor += (a + b) >> 1;
      else if (filtro === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        valor += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      atual[i] = valor & 0xff;
    }
    atual.copy(saida, linha * passo);
    anterior = atual;
  }
  const cores = new Set();
  const luminancias = [];
  for (let i = 0; i < saida.length; i += canais) {
    const r = saida[i];
    const g = canais >= 3 ? saida[i + 1] : r;
    const b = canais >= 3 ? saida[i + 2] : r;
    cores.add((r << 16) | (g << 8) | b);
    luminancias.push(0.299 * r + 0.587 * g + 0.114 * b);
  }
  const media = luminancias.reduce((s, v) => s + v, 0) / luminancias.length;
  const variancia =
    luminancias.reduce((s, v) => s + (v - media) ** 2, 0) / luminancias.length;
  return {
    largura,
    altura,
    cores: cores.size,
    desvio: Math.sqrt(variancia),
    bytes: dados.length,
  };
}

// ------------------------------------------------------------------ roteiro ---

async function capturar(arquivo) {
  const { imageData } = await pedir('GetSourceScreenshot', {
    sourceName: FONTE,
    imageFormat: 'png',
    imageWidth: 960,
    imageHeight: 540,
  });
  const base64 = String(imageData).replace(/^data:image\/png;base64,/, '');
  writeFileSync(`${SHOTS}/${arquivo}.png`, Buffer.from(base64, 'base64'));
  const info = lerPng(base64);
  const hash = createHash('sha256').update(base64).digest('hex').slice(0, 12);
  return { ...info, hash };
}

async function main() {
  await conectar();
  const lista = await pedir('GetSceneList');
  // O nome do campo mudou entre versões do obs-websocket: cai no antigo se o
  // novo não vier. Sem cena, não há onde criar a fonte.
  const cena = lista.currentProgramSceneName ?? lista.currentProgramScene;
  if (!cena) throw new Error('não achei a cena atual do OBS');
  const fonte = FONTE;
  const marca = Date.now();
  const morta = 'http://127.0.0.1:9/?t=1'; // porta 9: nada escuta

  // Rodada anterior interrompida pode deixar a fonte criada; recriar falharia.
  await pedir('RemoveInput', { inputName: fonte }).catch(() => {});

  await pedir('CreateInput', {
    sceneName: cena,
    inputName: fonte,
    inputKind: 'browser_source',
    inputSettings: { url: morta, width: 960, height: 540, shutdown: false },
    sceneItemEnabled: true,
  });

  const resultados = [];
  try {
    for (let rodada = 1; rodada <= 2; rodada++) {
      const capturas = [];
      for (const secao of [0, 1]) {
        const url = NEGATIVO ? morta : urlDa(secao, `${marca}-${rodada}-${secao}`);
        await pedir('SetInputSettings', {
          inputName: fonte,
          inputSettings: { url },
          overlay: true,
        });
        await new Promise((r) => setTimeout(r, 4000));
        const captura = await capturar(`rodada${rodada}-secao${secao}`);
        capturas.push(captura);
        console.log(
          `rodada ${rodada} seção ${secao}: ${captura.cores} cores, ` +
            `σ=${captura.desvio.toFixed(1)}, ${captura.bytes} B (${captura.hash})`,
        );
      }
      resultados.push({ rodada, capturas });
    }
  } finally {
    await pedir('RemoveInput', { inputName: fonte }).catch(() => {});
    ws.close();
  }

  // --------------------------------------------------------------- veredito ---
  const falhas = [];
  for (const { rodada, capturas } of resultados) {
    const [primeira, segunda] = capturas;
    if (primeira.cores < 50) {
      falhas.push(`rodada ${rodada}: projeção não renderizou (${primeira.cores} cores)`);
    }
    if (segunda.cores < 50) {
      falhas.push(`rodada ${rodada}: segunda seção não renderizou (${segunda.cores} cores)`);
    }
    if (primeira.hash === segunda.hash) {
      falhas.push(`rodada ${rodada}: trocar de seção não mudou o pixel`);
    }
  }

  if (falhas.length) {
    console.error(`REPROVADO: ${falhas.join('; ')}`);
    process.exit(1);
  }
  console.log('APROVADO: 2 rodadas, projeção renderizada e seção mudou o pixel');
}

main().catch((erro) => {
  console.error(`REPROVADO: ${erro.message}`);
  process.exit(1);
});
JS

passo "Rodando o piloto (duas rodadas)"
set +e
OBS_URL="ws://127.0.0.1:4455" \
OBS_SENHA="$OBS_SENHA" \
KENTRON_BASE="$BASE" \
KENTRON_SHOTS="$SHOTS" \
KENTRON_NEGATIVO="$NEGATIVO" \
  node "$DRIVER"
RC=$?
set -e

if [ "$NEGATIVO" = 1 ]; then
  # Prova negativa: o driver TEM de reprovar. Se aprovar, o gate é fraco.
  if [ "$RC" -eq 0 ]; then
    echo "REPROVADO: a prova negativa aprovou uma URL morta — o gate é fraco" >&2
    exit 1
  fi
  echo "APROVADO: a prova negativa reprovou a URL morta (rc=$RC)"
  exit 0
fi

[ "$RC" -eq 0 ] || { echo "o piloto reprovou (rc=$RC)" >&2; exit "$RC"; }
echo "capturas em $SHOTS"
