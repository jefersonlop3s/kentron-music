# Quadro de trabalho — quem está fazendo o quê, agora

> Único lugar onde uma sessão enxerga a outra. Caminho canônico: o deste arquivo **no diretório principal,
> na branch principal** — nunca a cópia de um worktree. Edite pelo `.agents/quadro.sh editar` e commite com
> `.agents/quadro.sh commit "docs(quadro): <o que mudou>"` (leva só este arquivo). Protocolo completo na skill
> `quadro-de-trabalho`.

## Como usar (3 momentos obrigatórios)

1. **Antes**: leia tudo; se alguém declarou os mesmos paths, combine em "Notas entre agentes". Acrescente sua entrada.
2. **Durante**: atualize ao mudar de repositório, branch ou escopo.
3. **Ao terminar**: mova para "Concluído recentemente" com PRs e o que ficou aberto; limpe as branches.

Timestamp sempre de `date -u '+%F %H:%M'`. **Paths sob edição** é o campo mais importante.

```markdown
### <agente>/<slug> — <título curto>
- **Início**: AAAA-MM-DD HH:MM (UTC) · **Última atualização**: AAAA-MM-DD HH:MM (UTC)
- **Repositórios**: ... · **Branches**: ... · **Worktrees**: ...
- **Demanda**: ...
- **Paths sob edição**: ...
- **Status**: em andamento | aguardando decisão do dono | em revisão | bloqueado
- **PRs**: ...
```

---

## 🔴 Em andamento

Nenhum.

## 🟡 Interrompido / órfão

Nenhum.

## 🟢 Concluído recentemente

### cline/de6b0 — Kentron: prova de navegador, locales e piloto automatizado
- **Início:** 2026-11-21 09:30 (UTC) · **Última atualização:** 2026-11-21 10:20 (UTC)
- **Repositório:** `kentron-music` · **Branches:** `feature/locales-pt-br` (de `main` `acf94f8`; integrada por merge `--ff-only`) · **Worktrees:** diretório principal
- **Demanda:** o dono respondeu as pendências (locale `pt_BR`, contrato do `musicxml`, piloto automatizado) e pediu para **ver e testar** o que foi entregue.
- **Paths sob edição:** nenhum (concluído).
- **Status:** concluído — `main` = `origin/main` = `302f7d0`, bateria verde no push.
- **PRs:** nenhum (merge `--ff-only` local).

**Feito:**
- **i18n:** `pt_BR` era a resposta, mas o `gen-l10n` exige arquivo-base por idioma — `pt_BR` sem `pt` aborta, e `es_419` é região UN M49 que ele não trata como país. Ficou `pt`/`en`/`es` com a correspondência explícita no `AGENTS.md`.
- **Contrato de arquitetura:** `musicxml` é **stub**, não entrega.
- **Prova de navegador** (Playwright/Chromium headless) sobre o build web: home, leitor, transposição, projeção, avanço/volta, contraste, bootstrap por URL e cópia do link.
- **Dois defeitos achados por olhar a tela:** ícones como "tofu" (`pubspec` sem `flutter: uses-material-design`) e AppBar do Modo Palco claro sobre corpo escuro. Ambos corrigidos, com teste.
- **ADR-0010:** portão R1-D vira **piloto automatizado** (VM Windows + OBS por Browser Source, duas rodadas simuladas); o uso ao vivo vem depois. ADR-0002/ADR-0009 emendados; ROADMAP e plano v2.6 alinhados.
- **`scripts/e2e-web.sh`** versiona a prova de fumaça (Playwright fora do repositório).

**Provado:** 30 testes de app; `flutter analyze` limpo; `flutter build web --release` verde; prova de navegador com clipboard contendo `…/?projecao=<base64>`; push com gitleaks, trivy, semgrep, types, test, lint, shell, deny, arb e flutter.

**Aberto:** prova **dentro do OBS** (roteiro do piloto automatizado); erro de console `wasm_bindgen` declarado duas vezes; setlists; alpha assistido.

### cline/de6b0 — Kentron: Browser Source do OBS por URL
- **Início:** 2026-11-21 03:10 (UTC) · **Última atualização:** 2026-11-21 03:35 (UTC)
- **Repositório:** `kentron-music` · **Branches:** `feature/browser-source-obs` (de `main` `789dd4f`; integrada por merge `--ff-only`) · **Worktrees:** diretório principal
- **Demanda:** transportar a projeção para fora do app; o próximo item do R1-B era o Browser Source do OBS.
- **Paths sob edição:** nenhum (concluído).
- **Status:** concluído — `main` = `origin/main` = `4bef4f3`, bateria verde no push.
- **PRs:** nenhum (merge `--ff-only` local).

**Feito:** `?projecao=<base64url do ChordPro>` abre o app direto na projeção; `chordproDaUrl`/`urlDaProjecao` fazem o round-trip; `ProjecaoApp` sobe sem tela inicial; o leitor oferece "Copiar link de projeção" no alvo web. Sem `obs-websocket`.

**Provado:** 5 testes de URL + 2 de bootstrap; suíte do app com **29 testes**; `flutter analyze` limpo; `flutter build web --release` compilou (`✓ Built build/web`); push com todos os gates verdes.

**Aberto:** o round-trip **dentro do OBS** não foi executado — o build prova compilação, não o Browser Source em uso. Tela secundária nativa segue sem transporte.

### cline/de6b0 — Kentron: modo projeção manual em tela cheia
- **Início**: 2026-11-21 02:45 (UTC) · **Última atualização**: 2026-11-21 03:05 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/modo-projecao` (de `main` `2e35d0c`; integrada por merge `--ff-only`) · **Worktrees**: diretório principal
- **Demanda**: continuar a R1-C depois do leitor; a projeção era o próximo item do caminho.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — `main` = `origin/main` = `97a484b`, bateria verde no push.
- **PRs**: nenhum (merge `--ff-only` local).

**Feito:** `ProjectionPage` com uma seção por vez, sem acorde, avanço por toque/teclado/rodapé; botão **Projetar** no leitor; chaves `stageProject`, `stageProjectNext` e `stageProjectPrevious` nos três ARBs-base. `ROADMAP`, `PENDING`, `CHANGELOG` e `CONCLUSION` atualizados.

**Provado:** `flutter analyze` limpo; **22 testes** de app passando (5 do modo projeção + 1 de abertura a partir do leitor); push com gitleaks, trivy, semgrep, types, test, lint, shell, deny, arb e flutter aprovados.

**Próximo passo:** transporte para tela secundária/janela do OBS e setlists; depois o ensaio assistido do R1-C.

### cline/de6b0 — Kentron: leitor de cifras (Modo Palco) entregue
- **Início**: 2026-11-21 02:00 (UTC) · **Última atualização**: 2026-11-21 02:40 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/leitor-cifras` (de `main` `12ca7fb`; integrada por merge `--ff-only`) · **Worktrees**: diretório principal
- **Demanda**: continuar a R1-C depois de o ROADMAP separar alpha de piloto; o próximo passo era o leitor de cifras.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — `main` = `origin/main` = `60dd439`, bateria verde no push.
- **PRs**: nenhum (merge `--ff-only` local).

**Feito:**
- `SongReaderPage` (Modo Palco): cifra inteira, acorde sobre a letra, transposição por tom e alternância de contraste; erro do núcleo visível.
- `chordLine` alinha acordes nas colunas do núcleo; acorde que invadiria o anterior é empurrado.
- Botão **Abrir cifra** na tela inicial; chaves novas `stageContrast`, `stageKey`, `stageLoadFailed`, `stageOpenReader` nos ARBs-base.
- `ROADMAP` R1-C: leitor marcado; setlists e projeção seguem abertas. `PENDING`, `CHANGELOG` e `CONCLUSION` atualizados.

**Provado:** `flutter analyze` limpo; 16 testes de app (5 herméticos do leitor + 3 provas reais na ponte + 8 anteriores); push com gitleaks, trivy, semgrep, types, test, lint, shell, deny, arb e flutter aprovados.

**Próximo passo:** projeção manual (tela secundária/OBS) e setlists no Modo Palco; depois o ensaio assistido do R1-C.

### cline/de6b0 — Kentron: ROADMAP corrigido, portão da R1 em dois
- **Início**: 2026-11-21 01:40 (UTC) · **Última atualização**: 2026-11-21 01:59 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/roadmap-portao-r1` (de `main` `25edaad`; integrada por merge `--ff-only`) · **Worktrees**: diretório principal
- **Demanda**: o dono pediu correção do ROADMAP e decidiu que o piloto pesado (dois cultos, zero perda, revisão por nativos) vai para o fim, para não travar o ciclo.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — `main` = `origin/main` = `c1c88ea`, bateria verde no push.
- **PRs**: nenhum (merge `--ff-only` local).

**Feito:**
- **ROADMAP corrigido:** `R1-0 · Fundação técnica` registra as entregas que só existiam na CONCLUSION (ponte FFI, alvos Android/iOS/Web, Native Assets, `HomeScreen`).
- **Portão em dois:** `R1-C · Alpha utilizável` (um ensaio assistido, feedback em dias) e `R1-D · Piloto e release` (dois cultos, no fim, depois do R1.1). Os critérios da ADR-0002 ficam intactos — o piloto foi reordenado, não removido.
- **ADR-0009 aceita:** decisão, custo declarado (alpha não substitui evidência de produção) e pressuposto de produto.
- **Plano v2.6 alinhado** (§3 e §5), `AGENTS.md` (release é R1-D), PENDING (alpha e piloto como itens separados), CHANGELOG e CONCLUSION.
- **Portão de Entrada da R1 corrigido:** era o spike S5, contradizendo a ADR-0005; agora o S5 dimensiona R1.1 e não bloqueia alpha nem piloto.

**Provado:** `verificar_refs.py`, `verificar_adr.py` e `documentation.py check` verdes; push com gitleaks, trivy, semgrep, types, test, lint, shell, deny, arb e flutter aprovados.

**Próximo passo:** leitor de cifras (Modo Palco) e projeção manual — são o que falta para o alpha da R1-C.

### cline/de6b0 — Kentron: contrato de validação com gitleaks
- **Início**: 2026-11-21 00:02 (UTC) · **Última atualização**: 2026-11-21 00:10 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/contrato-validacao` + `fix/semgrep-canario` (de `main` `99012e3`; integradas por merge `--ff-only` → `3d6b826`) · **Worktrees**: diretório principal
- **Demanda**: o dono decidiu que a bateria inteira **não** roda em toda invocação de validação e que derrubar emulador como gate é perigoso; o que vale como validação real é o gitleaks (mais o CI completo no `pre-push`).
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — `main` = `origin/main` = `66eacf1`, bateria verde no push.
- **PRs**: nenhum (integração por merge `--ff-only`).

**Feito:**
- **`.harness.toml` `[validation]`:** `configured = true` com um gate explícito — `gitleaks`, mesmo argv do `gate_gitleaks` do `pre-push`. O contrato recusava push por desenho enquanto estivesse vazio; agora declara a validação barata e real. A bateria completa continua no `pre-push`, onde já era aplicada.
- **Higiene de dispositivos:** registrado na skill que **não** vira gate — derrubar dispositivo dentro de um comando de verificação é destrutivo para quem estiver com `flutter run`.
- **`origin/HEAD` para `main`:** o GitHub já tinha `main` como default; o ref **local** `refs/remotes/origin/HEAD` é que apontava para `master`. Corrigido com `git remote set-head origin -a`, sem mudança no GitHub.

**Provado:**
- `validation.py check` → `gates: gitleaks`, `80 commits scanned`, `no leaks found`, saída 0.
- Contrato no sentido negativo: gate trocado por `false` → `gates(s) reprovado(s): gitleaks`, saída 1 (arquivo restaurado depois).
- Gitleaks no sentido negativo: repositório temporário com chave de formato AWS → `leaks found: 1`, saída 1.

**Ficou aberto:**
- O `semgrep` bloqueou o primeiro push porque o literal do canário estava no `CHANGELOG`/`CONCLUSION`. Reescrito sem o valor; o achado está registrado nos dois documentos.
- Higiene de dispositivos segue como ritual + `scripts/encerrar-dispositivos.sh`, sem gate.

### cline/de6b0 — Kentron: perfil do projeto e shellcheck como gate
- **Início**: 2026-11-20 23:52 (UTC) · **Última atualização**: 2026-11-21 00:00 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/perfil-e-shellcheck` (de `main` `d5f2bbf`; integrada por merge `--ff-only` → `310199a`) · **Worktrees**: diretório principal
- **Demanda**: o dono pediu para ajustar o que ficou em aberto na sessão anterior — placeholders do `AGENTS.md`, `shellcheck` no script novo e a higiene de dispositivos como gate.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — `main` = `origin/main` = `310199a`, bateria verde no push.
- **PRs**: nenhum (integração por merge `--ff-only`).

**Feito:**
- **`AGENTS.md` sem placeholders:** "Perfil do projeto" declara topologia (`repository`), fronteiras (Rust + Flutter + `core_l10n` + Python), `main` protegida com `origin` único, stack com versões pinadas, comandos reais de validação, locales (`en`, `es`, `es_ES`, `pt`, `pt_PT`) e autoridades de aprovação; "Comunicação" fixa português (Brasil); "Release" diz o que existe (versão 0.0.0, sem release publicada).
- **`shellcheck` virou gate em dois pontos:** `scripts/ci-local.sh` (todos os `scripts/*.sh`, com `require` fail-loud) e `.githooks/pre-push` (só os `.sh` do push, mesmo recorte dos gates de lint por stack); o workflow `ci.yml` instala a ferramenta.
- **SC2164 corrigido no hook:** `cd "$REPO_ROOT" || exit 1` — sem isso, um `cd` que falhasse deixava os gates rodando no diretório errado, em silêncio.
- **Validadores verdes:** `verificar_refs.py` (estava quebrado com 7 referências, 3 delas minhas), `verificar_adr.py` e `documentation.py check`.

**Provado no sentido negativo:** com um `.sh` defeituoso injetado, o hook reprovou (`❌ shell`, `HOOK_EXIT=1`) e o log diagnóstico mostrou o achado; restaurado, o mesmo hook passou. No push real, `✅ shell` apareceu na bateria.

**Ficou aberto:**
- Higiene de dispositivos **não** virou gate: só vira se voltar a custar sessão — encerrar dispositivo durante um push é efeito colateral num comando de verificação.
- `.harness.toml` `[validation]` segue `configured = false`/`gates = []` embora o projeto tenha gates; `validation.py` recusa rodar assim. Decisão do dono, porque rodar a bateria inteira por push tem custo.

### cline/de6b0 — Kentron: lições da sessão viram skills
- **Início**: 2026-11-20 23:20 (UTC) · **Última atualização**: 2026-11-20 23:51 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/skills-sessao` (de `main` `5582d2f`; integrada por merge) · **Worktrees**: `~/.cline/worktrees/de6b0/app-music`
- **Demanda**: o dono autorizou remover a branch `spike/native-assets` e pediu que as lições da sessão anterior virassem skills do projeto.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído.
- **PRs**: nenhum (integração por merge `--ff-only`).

**Feito:**
- **Quatro skills novas** em `.agents/skills/`, com gatilho na tabela do `AGENTS.md`: `empacotamento-rust-flutter` (Native Assets, predicados técnicos e erros com assinatura), `higiene-de-dispositivos` (encerrar emulador/simulador depois da prova), `lote-nao-e-sequencia` (comando agrupado roda em paralelo) e `toolchain-unica` (duplicata de binário decide o PATH).
- **Mecanismo, não só prosa:** `scripts/encerrar-dispositivos.sh` detecta, encerra e verifica Android/iOS; `--dry-run` mostra o que faria; sai 1 se algo sobrar. Provado no sentido negativo com o emulador `kentron` no ar (`emulator-5554` detectado, encerrado, `adb devices` vazio).
- **Skills existentes corrigidas:** `vies-de-plausibilidade` §11 (o diff tem direção e o olho lê ao contrário), `toolchain-unica` (o cask que resolve o SDK root para si mesmo) e `quadro-de-trabalho` (ordem do commit do quadro).
- **Branches mergeadas removidas** com autorização do dono: `spike/native-assets` (local e `origin`), `feature/alvo-android`, `feature/alvo-ios`, `feature/alvo-web` e `feature/ponte-rust-flutter`.
- **Dispositivos encerrados:** `adb devices` e `xcrun simctl list devices booted` vazios ao fim da sessão.

**Ficou aberto:**
- Skill é julgamento, não gate: só a higiene de dispositivos tem mecanismo, e ele depende de alguém rodar o script. Se isso voltar a custar sessão, vira gate no `pre-push`.

### cline/de6b0 — Kentron: acesso remoto, host de build e alvo web
- **Início**: 2026-11-20 17:35 (UTC) · **Última atualização**: 2026-11-20 22:20 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `spike/native-assets` (= `main` `0da5011`, integrada por merge; apagar quando o worktree sair) · **Worktrees**: `~/.cline/worktrees/de6b0/app-music`
- **Demanda**: destravar o acesso remoto aos alvos, preparar o host de build e provar o app com o núcleo real.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — o núcleo Rust roda nas **quatro** plataformas (teste, navegador, Android, iOS) e o empacotamento migrou para Native Assets (ADR-0008 aceita, merge `0da5011`).
- **PRs**: nenhum (integração por merge `--ff-only`).

**Feito:**
- **Acesso remoto:** este Mac rodava a variante **OSS do Tailscale** (`tailscaled` do Homebrew, sem Network Extension) — que, segundo a própria documentação, **anuncia rotas mas não as usa**. Trocada pelo cask `tailscale-app` (Standalone/System Extension): a rota `192.168.6.0/24` apareceu em `utun0` e `ssh workspace` conecta direto. A hipótese anterior (NextDNS) foi **descartada por medição**.
- **Salto no `~/.ssh/config`** (aditivo; backup em `/tmp/ssh-config.backup-20261120T173755Z`): `Match originalhost … exec "! nc …"` só entra quando a LAN não responde e **autodesativou** depois da troca. Achado de sintaxe registrado: `Match host` casa com o nome **resolvido**; o apelido exige `originalhost`.
- **`lab` (`runner-linux`, 6 vCPU · 16 GiB · 85 G livres):** Rust 1.100.0 + alvos Android; imagem `ghcr.io/cirruslabs/flutter:stable` medida por dentro (Flutter 3.44 · Java 21 · Android SDK em `/opt/android-sdk-linux`, **sem NDK e sem Rust**).
- **Alvo web:** scaffold, wasm da ponte (`kentron_ffi_bg.wasm` 104 KB) e `scripts/servir-web.py` (COOP/COEP + `application/wasm`); build e serviço verificados nos dois sentidos — commit `2fec427`. Depois o servidor passou a escutar **só o local** por padrão (`--host` para expor).
- **Tela ligada à ponte:** `HomeScreen` chama `parseSong` e mostra título/tom/seções/acordes + `núcleo: kentron_ffi …`; o carregador é injetável, então os testes de widget seguem herméticos. Prova com o **cdylib real** em `bridge_test.dart`: `flutter test` 8/8, `analyze` limpo, `ci-local.sh` EXIT=0.
- **Achados:** `pumpAndSettle` estoura com a ponte real (o indicador anima sempre e a resposta chega por porta do FFI, fora do relógio falso) → espera em tempo real com `runAsync`; e o parser extrai acordes de linhas só com acordes.
- **Android entregue e rodando:** `android/` criado (`--org com.kentron`; appId `com.kentron.kentron_app`), `rust_builder` do cargokit ligado ao nosso crate fora do app (`--rust-crate-dir ../../crates/kentron_ffi`) com `--no-write-lib`, que protegeu o `main.dart` do código de exemplo; dependência no `pubspec`. `flutter build apk --debug` verde, com **`libkentron_ffi.so` dentro do APK** em três ABIs; no emulador `kentron` o app abre (COLD, 1,7 s), o processo vive sem `UnsatisfiedLink`, e o print — que **eu li e conferi** (leio imagens) — mostra `Exemplo`, `tom C · 2 seção(ões)`, os acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`.
- **Java 27 quebrava o build:** `Unsupported class file major version 71` no Gradle 9.5; resolvido com **JDK 21** (`brew install openjdk@21`, formula — sem sudo) + `flutter config --jdk-dir`. O Flutter já avisava que 27 era "newer than known valid".
- **Falha minha, registrada:** li um `git diff` vazio como "o pubspec não ganhou a dependência" quando o diff era contra o `HEAD`, não contra o backup — o `integrate` havia feito o `pub add`. Bastava ter grepyado o arquivo em vez de concluir pelo diff.
- **iOS entregue e rodando:** `ios/` criado (`--org com.kentron`; bundle id `com.kentron.kentronApp`, `pubspec` idêntico). O build falhou com `Unable to find bundle for dynamic library` — e a correção saiu de **ler o código do cargokit**: ele usa o `.a` **se existir** (`if (staticLibs.isNotEmpty)`) e só exige Framework quando não existe; como o crate declarava só `cdylib`, não havia `.a` para iOS. Bastou **`staticlib` no `crate-type`** (Android segue no `cdylib`). Prova: build do simulador verde (Xcode 27.2), app aberto (PID 53043) e o print — que eu li e conferi — com `Exemplo`, `tom C`, os acordes e `núcleo: kentron_ffi 0.0.0`. **Regressão do Android revalidada** após a mudança.
- **Risco anotado (e depois resolvido):** o Flutter avisava que o plugin `kentron_ffi` **não suporta Swift Package Manager** e que isso *viraria erro* numa versão futura (o cargokit é CocoaPods-only). Resolvido pela migração para Native Assets (ADR-0008): sem plugin, o aviso some por construção.
- **Manutenção:** removido o runtime de simulador redundante (`24B91016s`, seed, 7,7 GB) — ficou o `24B8117`, com o iPhone 17 das provas. E o **SPM** foi investigado com medição: desligar `--no-enable-swift-package-manager` **não** silencia o aviso (build verde, aviso presente) → hipótese minha derrubada por medição e config **restaurada** ao default com `--clear-features` (preservando `android-sdk`/`jdk-dir`). O frb 2.13.0 estável não tem SPM (nem opção no `integrate`, nem `Package.swift`); upstream as issues #3036/#3202 estão fechadas → o caminho durável é subir o gerador quando houver estável com SPM.
- **Lição de método (a que vale para a próxima sessão):** quando o erro está numa ferramenta de terceiro, **ler o código dela** deu a correção exata em minutos, enquanto tentativa e erro custaria builds de 5 min cada.
- **Prova no navegador (feita pelo dono):** a tela web mostrou os dados do núcleo — `Exemplo`, `Kentron · tom C · 2 seção(ões)`, acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`. Cadeia Rust → wasm → Flutter web → tela provada.
- **Toolchain Android medida e funcional:** SDK 36 completo em `~/Library/Android/sdk` (build-tools 36.0.0, emulator 37.3.10, **NDK 28.2.13676358**, platforms 36, imagem arm64-v8a), Java Temurin 27, AVD **`kentron`** criado, `flutter doctor` ✓. Versões tiradas do próprio SDK (não de memória).
- **Armadilha do `avdmanager`:** o do cask resolve o SDK root para o cask (sem system-images) → `Package path is not valid ... null`; e symlink não engana essa detecção. O SDK root só fica íntegro com **cópia real** de `cmdline-tools` dentro dele — o que também o deixa imune a `brew upgrade`.
- **Duas falhas minhas, registradas:** (1) paralelizei a cópia do `cmdline-tools` com o `avdmanager` que dependia dela e li a falha como bug da ferramenta; (2) **culpei o Flutter do brew** pela reprovação do gate sem provar — a reprodução isolada deu 5/5 e o teste completo 8/8. A causa era **contenção**: rodei a bateria enquanto baixava vários GB do SDK. A mesma revisão passou com a máquina livre.
- **Migração para Native Assets (ADR-0008 aceita, `bb44789`):** o Rust deixou de entrar como plugin cargokit e passou a **code asset** por build hooks; `rust_builder/` (65 arquivos) removido, `ios/` sem CocoaPods e sem `Podfile`, toolchain pinada em 1.97.1 **e copiada dentro do crate** (o `native_toolchain_rust` valida naquele caminho). Builds verdes e app **rodando** no iOS e no Android com o code asset; `flutter test` 8/8 e `ci-local.sh` EXIT=0. Fecha o risco de SPM por construção: sem plugin, não há incompatibilidade a corrigir.

**Ficou aberto:**
- **Toolchains duplicadas:** resolvido por remoção — o cask do Flutter saiu (fica só `~/development/flutter`) e o cask `android-platform-tools` também (fica só o `adb` do SDK).
- **Disco:** **25 GB livres** — apertado para builds de iOS; `~/Library/Developer/Xcode/DerivedData` e `apps/kentron_app/build/` são os candidatos de limpeza.
- **Máquinas de pé:** emulador Android (`emulator -avd kentron`) e simulador iOS (iPhone 17 bootado) — encerrar quando não precisar.
- **Ver no navegador / Android / iOS:** ✅ todos feitos.
- Instalações: NDK + Flutter pinado na `lab` (build Android por Docker), SDK/JDK/NDK no Mac (emulador) e Xcode (iOS — só o dono, via App Store). Matriz de versões medida na fonte: compileSdk/targetSdk **36**, minSdk **24**, **NDK 28.2.13676358**, Gradle **9.5.0**, AGP **9.3.1**, KGP **2.4.20**, **JDK ≥ 17**.

### cline/de6b0 — Kentron: higiene do CI, core_l10n integrado e ponte (ADR-0007)
- **Início**: 2026-11-20 14:45 (UTC) · **Última atualização**: 2026-11-20 15:11 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/ponte-rust-flutter` (= `main` `b8d4cc8`, integrada; apagar junto com o worktree) · **Worktrees**: `~/.cline/worktrees/de6b0/app-music`
- **Demanda**: higienizar (branch de escopo + matriz do CI duplicada), integrar o `core_l10n` e entregar a ponte Rust↔Flutter.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído — ponte integrada na `main` e empurrada (`04e9ea5`).
- **PRs**: nenhum (repositório local; integração por merge `--ff-only`).

**Feito:**
- Branch de escopo `feature/song-bundle-v1` apagada (idêntica à `main`, nome de escopo); trabalho em `feature/ponte-rust-flutter`.
- Matriz do CI com fonte única: o workflow instala as ferramentas e chama `scripts/ci-local.sh` — 4 jobs viraram 1, mesma cobertura, sem ação nova.
- `core_l10n` integrado ao app: `lib/core_l10n.dart` como caminho único de import, `MaterialApp` com `CoreL10n.delegate` + `...GlobalMaterialLocalizations.delegates`, ARB `homeWelcome` (en/pt/es) na tela inicial e 3 testes de widget (en, pt, es e fallback `pt_PT` → `pt`).
- Pipeline: geração do `core_l10n` no lugar e antes dos passos do app; o gate de Flutter do pre-push também gera.
- Achado medido e registrado: as `Global*` do `flutter_localizations` que o `gen-l10n` emite não satisfazem o `MaterialLocalizations` do `material_ui`; a "migração do `core_l10n`" do PENDING depende do gerador, não do `pubspec`.

**Pendente desta sessão:**
- Diretórios de plataforma do app (`flutter create --platforms=...`): sem eles a ponte roda em teste, mas o app não abre fora dele.
- Documentar no README o pré-requisito de **regerar** os bindings (`flutter_rust_bridge_codegen` 2.13.0 + `cargo fmt --all` depois).

**R1 · Ponte Rust↔Flutter (entregue):**
- ADR-0007 **aceita pelo dono**; `flutter_rust_bridge_codegen` 2.13.0 instalado por `cargo install` (ferramenta de sistema, autorizada pelo aceite).
- Novo crate adaptador `crates/kentron_ffi` — único `cdylib` e único a depender do `flutter_rust_bridge` — expõe `parse_song(input) -> SongSummary` ao Dart; bindings gerados e **versionados** em `crates/kentron_ffi/src/frb_generated.rs` e `apps/kentron_app/lib/src/rust/`.
- `notation_bundle` volta a ser domínio puro, com `unsafe_code = "forbid"` intacto.
- **Emenda medida à ADR-0007:** a cláusula "`forbid` permanece" era inalcançável — a cola gerada tem um bloco `unsafe` e `forbid` não aceita `allow`. O adaptador usa `deny` com `#[allow(unsafe_code)]` escopado ao módulo gerado; o invariante que o dono quis preservar segue de pé onde o código é nosso.
- **Prova de ponta a ponta:** `apps/kentron_app/test/bridge_test.dart` carrega o cdylib e chama o Rust (resumo real; cifra vazia vira exceção). 58 testes Rust + 5 de Flutter verdes.
- Doc de arquitetura atualizado: o crate adaptador entrou na decomposição e o "estado atual" deixou de dizer que `core_l10n`/ponte não estão integrados.
- Dois bloqueios reais que viraram correção: o código gerado não é `rustfmt`-limpo na edição 2024 (o pipeline reprovou até formatar) e dependência de caminho sem `version` é lida como wildcard pelo `cargo deny`. Ambos registrados.

### cline/de6b0 — sovereign-harness 0.7.2: duas lições do kentron
- **Início**: 2026-11-20 13:56 (UTC) · **Última atualização**: 2026-11-20 14:10 (UTC)
- **Repositório**: `sovereign-harness` (personal-digital-sovereignty) · **Branches**: `main` (`4066f5a`, tag `v0.7.2`) · **Worktrees**: nenhum (temporário removido)
- **Demanda**: publicar no kit as duas lições nascidas no kentron e cortar a release 0.7.2.
- **Paths sob edição**: nenhum (concluído).
- **Status**: concluído.
- **PRs**: [#21](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/21) e [#22](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/22), ambos mergeados.

**Feito:**
- PR #21 → skill `estado-compartilhado-multi-agente` §1c (branch de escopo vira segundo trunk), com o caso medido no kentron.
- PR #22 → gate de lint lê a edição do `Cargo.toml` em vez de fixar 2021; teste novo provado no negativo; versões → 0.7.2 nos 7 manifestos do kit.
- Tag anotada `v0.7.2` + release publicada; CI verde (ubuntu, macos, tipos do OpenCode).
- Bateria local: 99 testes OK, `test_quadro`, `merge_gate_base` (5/5) e `test_ref_main` verdes; `sincronizar --verificar` 45/45.
- Worktree temporário e as duas branches (a do fix e a do PR #21) apagadas.

**Ficou aberto:**
- O clone do marketplace nesta máquina segue em `2e654e2` (0.7.1): atualizar com `git pull --ff-only` é decisão do dono.
- Canário Claude live segue pendente de `claude login` (documentado no kit, não bloqueia).

### cline/de6b0 — R1 · Song Bundle v1: modelo e persistência
- **Início**: 2026-11-20 12:28 (UTC) · **Última atualização**: 2026-11-20 13:42 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/song-bundle-v1` (= `main` `a68c85c`) · **Worktrees**: `~/.cline/worktrees/de6b0/app-music`
- **Demanda**: Song Bundle v1 — o dono decidiu começar pelo modelo em memória; com o modelo provado, aceitou a ADR-0006 e a persistência entrou.
- **Paths sob edição**: nenhum (sessão encerrada).
- **Status**: concluído.
- **PRs**: nenhum (integração por merge `--ff-only` na `main`).

**Feito:**
- Modelo em memória (`bundle`): `SongId`, `SetlistId`, `SongBundle`, `Library`, `Setlist` e `BundleError` com `message_key()`; bibliografia vem do metadata do `.cho`, sem duplicação.
- Persistência (`bundle_store`, ADR-0006 **aceita**): diretório com `meta.json` versionado + `song.cho`, `library.json`, setlist em JSON e exportação por cópia do diretório.
- `chordpro::write` fecha o ciclo do parser com saída determinística; ida e volta pelo parser preserva o modelo.
- Migração por degrau com cadeia vazia (v1 é o primeiro esquema): versão sem degrau é recusada em voz alta — provado com 0 e 2 recusados, 1 aceito.
- Primeira dependência externa do workspace (`serde`/`serde_json`, **11 pacotes medidos** no `Cargo.lock`), aprovada por `cargo deny`.
- 26 testes novos no total (14 do modelo + 12 da persistência); workspace de 29 para 55; `ci-local.sh` verde.
- Branches substituídas apagadas (`feature/R1-restante`, `feature/R1-transposicao-harmonia`, `wip/harmony-transpose`).
- Achado no push: o gate de lint do pre-push formatava com `rustfmt --edition 2021` fixo contra um workspace em edition 2024 — os dois gates do projeto discordavam. Corrigido no commit `a68c85c` (edição lida do `Cargo.toml`), provado nos dois sentidos.

**Ficou aberto:**
- Empacotar `score.musicxml` e `tabs/*.atex` no bundle: a ADR-0006 já os prevê, o v1 escreve só o `.cho`.
- Contêiner de arquivo único (zip) para exportação — v1.1 da ADR-0006.
- Integração Flutter (ponte real) e UI de leitor/setlists; exportação OpenLyrics/Holyrics; projeção; piloto.

### cline/de6b0 — R1 · Transposição e harmonia (Etapa 1)
- **Início**: 2026-11-20 03:35 (UTC) · **Última atualização**: 2026-11-20 04:45 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main` (`29e2bda`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: entregar a Etapa 1 da R1 (transposição) e trazer o rascunho de `wip/harmony-transpose` para a `main`.
- **Paths sob edição**: nenhum.
- **Status**: concluído.
- **PRs**: nenhum.

**Feito:**
- `Chord::transpose` em `chord_naming.rs`: raiz e baixo movidos por letra e acidente, com fallback enarmônico quando a grafia diatônica passaria de dois acidentes.
- `harmony.rs` reintroduzido: `HarmonyEngine` transpoe a música inteira (incluindo a metadata `key`) e renderiza em Common, Latin, German e Nashville.
- Testes: 29 no workspace; unitários de transposição, integração do motor e dourados novos (`golden_harmony/worship.cho` + saídas Common e Latin).
- Pipeline local (`./scripts/ci-local.sh`) verde, `exit 0`, antes do commit.
- Integração por merge `--ff-only` em `main` (`8f596f4` código, `29e2bda` docs).
- Documentos vivos atualizados: README, ROADMAP, PENDING, CONCLUSION, CHANGELOG, plano v2.6 e ADR-0005 (proposta).

**Ficou aberto:**
- **ADR-0005 rejeitada** (2026-11-20): vale a ADR-0002 integral. MusicXML/ATEX seguem no marco R1.1 como ordem de execução, não como corte de escopo.
- Fixtures de parser para acordes invertidos, `6/9` e acidentes duplos (a nomeação já os cobre; falta provar a preservação na leitura do `.cho`).
- Branches substituídas apagadas nesta sessão: `feature/R1-restante`, `feature/R1-transposicao-harmonia` e `wip/harmony-transpose` — nenhuma com conteúdo exclusivo em relação à `main`.

### cline/de6b0 — Rascunho de harmonia (WIP)
> Entrada encerrada: absorvida pela entrada acima. Mantida como registro histórico. A branch `wip/harmony-transpose` (`35971c4`) está inteiramente contida na `main` e pode ser apagada após confirmação do dono.

### cline/de6b0 — CI: versão fixada e core_l10n no pipeline
- **Início**: 2026-11-20 04:25 (UTC) · **Última atualização**: 2026-11-20 04:45 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main` (`e211ae7`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: fechar as pendências do replanejamento: versão do Flutter, `core_l10n` no pipeline e risco de *drift*.
- **Paths sob edição**: nenhum.
- **Status**: concluído.
- **PRs**: nenhum.

**Feito:**
- `.flutter-version` (3.50.1) como fonte única; o script falha se a versão instalada divergir (provado no sentido negativo).
- `packages/core_l10n` coberto com `gen-l10n` + `analyze --no-fatal-infos` em cópia temporária.
- Workflow espelha a versão fixada e cobre `core_l10n`.
- Localizações geradas (`app_localizations*.dart`) no `.gitignore`.
- Pipeline local verde (`exit 0`) após as mudanças.

**Ficou aberto:**
- Sincronia entre script e workflow continua manual (risco de *drift* registrado em `PENDING`).
- Migração de `core_l10n` de `flutter_localizations` para `material_ui` (dois `info` de depreciação).

### cline/de6b0 — CI local-first (ADR-0004)
- **Início**: 2026-11-20 04:05 (UTC) · **Última atualização**: 2026-11-20 04:20 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main` (`9051b5b`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: replanejar as execuções de pipeline para o modelo local-first, com o Actions como rede de segurança.
- **Paths sob edição**: nenhum.
- **Status**: concluído.
- **PRs**: nenhum.

**Feito:**
- ADR-0004 aceita e registrada no índice.
- `scripts/ci-local.sh` criado e verde de ponta a ponta (Rust, `cargo deny`, ARBs, Flutter `analyze`/`test`/`gen-l10n`).
- `.githooks/pre-push` com gates transversais `deny`, `arb` e `flutter`.
- `.github/workflows/ci.yml` sem gatilho em `push`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.
- Documentos vivos atualizados: README, ROADMAP, PENDING, CONCLUSÃO, CHANGELOG e plano v2.6.
- Dois pushes (`3a8dc58`, `9051b5b`) sem disparar Actions — confirmado em `gh run list`.

**Ficou aberto:**
- Matriz duplicada entre script e workflow; risco de *drift* registrado em `PENDING`.
- Versão do Flutter não fixada em nenhum dos dois.
- `packages/core_l10n` sem `analyze` no pipeline local.

### cline/de6b0 — R1 · Cifra e projeção (branch ativa)
- **Início**: 2026-11-19 21:31 (UTC) · **Última atualização**: 2026-11-20 03:44 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/R1-cifra-projecao` integrada à `main` (`7460f57`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: entregar a R1 (cifra local, projeção manual) e consolidar o parser na `main`.
- **Paths sob edição**: nenhum.
- **Status**: concluído — branch integrada e a apagar; o restante da R1 começa em nova branch.

**Feito:**
- `analysis_options.yaml` commitado em `91de756` (exclusão de `build/**` no analyzer).
- `feature/R1-cifra-projecao` integrada por merge `50c7323`.
- Documentação atualizada em `7460f57`.
- Validação do resultado integrado: 15 testes Rust, Clippy, formatação, `check_arb.py`, `cargo deny`, `flutter analyze` e `flutter test` verdes.

**Ficou aberto:**
- O restante da R1 (transposição, persistência, UI de palco, projeção) deve começar em nova branch a partir da `main`.
- `wip/harmony-transpose` permanece como rascunho local, aguardando decisão.

### cline/de6b0 — Análise dos planos, R0 publicado e R1 iniciada
- **Início**: 2026-11-19 21:31 (UTC) · **Última atualização**: 2026-11-20 03:13 (UTC)
- **Repositórios**: `kentron-music` (github.com/jefersonlop3s/kentron-music)
- **Branches**: `main` (`d21d1fe`), `feature/R1-cifra-projecao` (`4beb085`), `wip/harmony-transpose` (`35971c4`, local, não publicada) · **Worktrees**: `/Users/jefersonlopes/.cline/worktrees/de6b0/app-music`
- **Demanda**: validar e mitigar os planos anteriores, publicar o R0 na `main` e iniciar a R1 com bateria de testes.
- **Paths sob edição**: nenhum (sessão encerrada).
- **Status**: concluído
- **PRs**: sem PR; integração por *fast-forward* na `main`.

**O que ficou aberto:**
- Flutter não instalado neste ambiente; `flutter analyze`/`flutter test` rodaram apenas no CI.
- `core_l10n` não integrado ao app; ponte Rust ↔ Flutter é placeholder.
- R1 parcial: parser ChordPro e placeholder MusicXML; sem transposição, persistência, UI de palco ou projeção.
- `wip/harmony-transpose` não compila e não foi publicada; reimplementar `Chord::transpose` antes de mesclar.

**Hipóteses refutadas:**
- "R0 commitado e enviado": falso — os commits estavam apenas em `cline/de6b0`, a `main` local seguia em `fd7d588` e o remoto só tinha `master`.
- "Commit direto na `main` é permitido": o guard bloqueia; o fluxo correto é branch + merge *fast-forward*.
- "A validação local de Flutter já ocorreu": não ocorreu; apenas o CI executou `flutter analyze`/`flutter test`.

### cline/de6b0 — Instalação do Flutter e validação local
- **Início**: 2026-11-20 03:20 (UTC) · **Última atualização**: 2026-11-20 03:35 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main`
- **Demanda**: instalar Flutter 3.50.x e fechar a pendência de validação local do R0.
- **Paths sob edição**: nenhum.
- **Status**: concluído.

**Feito:**
- Flutter 3.50.1 (stable) instalado em `~/development/flutter` (clone parcial, canal stable).
- `flutter analyze` no app: sem achados. `flutter test`: teste de fumaça passando.
- `flutter gen-l10n` em `packages/core_l10n`: geração sem erro; artefatos gerados removidos por serem reproduzíveis.
- Documentos atualizados: README, ROADMAP, PENDING, CONCLUSION, CHANGELOG e plano v2.6.

**Ficou aberto:**
- O SDK não está no `PATH` padrão; usar `~/development/flutter/bin/flutter` ou exportar o caminho.
- CI ainda não analisa `packages/core_l10n`; item registrado em PENDING.
- `apps/kentron_app/analysis_options.yaml` foi assumido e commitado em `91de756` (exclusão de `build/**` no analyzer).

## Notas entre agentes

- Flutter 3.50.1 instalado em `~/development/flutter` (não está no `PATH` padrão). Para usar: `export PATH="$HOME/development/flutter/bin:$PATH"`.
- Remoção de worktrees validada em 2026-11-20 03:13 (UTC): o worktree `de6b0` está limpo; os diretórios `40e55` e `b5088` em `~/.cline/worktrees` estão vazios. Nenhum stash pendente. O rascunho de harmonia está na branch local `wip/harmony-transpose`, que sobrevive à remoção do worktree.
- Branch de escopo vira segundo trunk: `feature/R1-restante` era ancestral da `main` e continuava se apresentando como "o trabalho da R1" — o custo medido foi uma sessão lendo `git status` no checkout errado. Nome de branch = unidade de trabalho; ao entregar, apague a branch no mesmo passo (skill `estado-compartilhado-multi-agente` §1c).
