//! Prova a ponte Rust ↔ Flutter de ponta a ponta (ADR-0007).
//!
//! Exige o `cdylib` construído — na raiz do repositório:
//!
//! ```text
//! cargo build -p kentron_ffi
//! ```
//!
//! O teste carrega o artefato **explicitamente**: a heurística de caminho do
//! `flutter_rust_bridge` assume um layout `rust/` na raiz do app, e aqui o crate
//! vive em `crates/kentron_ffi` com o `target/` do workspace.

import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/main.dart';
import 'package:material_ui/material_ui.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/rust/frb_generated.dart';

/// Caminho do `cdylib` no `target/debug` do workspace.
String _libraryPath() {
  final name = Platform.isMacOS
      ? 'libkentron_ffi.dylib'
      : Platform.isWindows
      ? 'kentron_ffi.dll'
      : 'libkentron_ffi.so';
  final file = File('../../target/debug/$name');
  if (!file.existsSync()) {
    fail(
      'cdylib ausente em ${file.absolute.path}. '
      'Construa antes: cargo build -p kentron_ffi',
    );
  }
  return file.absolute.path;
}

void main() {
  setUpAll(() async {
    await RustLib.init(externalLibrary: ExternalLibrary.open(_libraryPath()));
  });

  test('parse_song atravessa a ponte e devolve o resumo', () async {
    final summary = await parseSong(
      input:
          '{title: Santo}\n{key: C}\n{artist: Autor}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]santo\n',
    );

    expect(summary.title, 'Santo');
    expect(summary.key, 'C');
    expect(summary.artist, 'Autor');
    expect(summary.sections, 1);
    expect(summary.chords, ['C', 'G', 'Am']);
    expect(summary.core, startsWith('kentron_ffi '));
  });

  test('cifra vazia vira exceção, não resumo vazio', () async {
    await expectLater(
      parseSong(input: '   \n'),
      throwsA(predicate((Object error) => '$error'.contains('vazia'))),
    );
  });

  // A prova que faltava: a TELA renderiza o que o núcleo Rust devolveu, com o
  // cdylib real carregado — o caminho inteiro, não um dublê.
  //
  // `pumpAndSettle` não serve aqui: o `CircularProgressIndicator` anima
  // indefinidamente e a resposta do núcleo chega por uma **porta do FFI**, que
  // o relógio falso do teste não avança. Daí a espera em tempo real via
  // `runAsync`, até o indicador sair da árvore.
  testWidgets('a tela inicial mostra o que o núcleo Rust devolveu', (
    tester,
  ) async {
    await tester.pumpWidget(const KentronApp(locale: Locale('en')));

    for (var tentativa = 0; tentativa < 40; tentativa++) {
      if (find.byType(CircularProgressIndicator).evaluate().isEmpty) {
        break;
      }
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 25)),
      );
      await tester.pump();
    }
    expect(
      find.byType(CircularProgressIndicator),
      findsNothing,
      reason: 'o núcleo não respondeu a tempo',
    );

    expect(find.text('Exemplo'), findsOneWidget);
    expect(find.text('Kentron · tom C · 2 seção(ões)'), findsOneWidget);
    for (final acorde in <String>['C', 'G', 'Am', 'F']) {
      expect(find.text(acorde), findsOneWidget, reason: 'acorde $acorde');
    }
    expect(find.textContaining('núcleo: kentron_ffi '), findsOneWidget);
  });

  test('read_song devolve a estrutura linha a linha', () async {
    final detail = await readSong(
      input:
          '{title: Santo}\n{key: C}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]santo\n',
    );

    expect(detail.title, 'Santo');
    expect(detail.key, 'C');
    expect(detail.sections, hasLength(1));
    final linha = detail.sections.first.lines.first;
    expect(linha.text, 'Santo, santo');
    expect(
      linha.chords.map((ChordMark mark) => mark.chord).toList(),
      <String>['C', 'G'],
    );
    expect(linha.chords.first.position, 0);
    expect(linha.chords.last.position, 7);
    expect(detail.core, startsWith('kentron_ffi '));
  });

  test('transpose_song muda o tom e os acordes', () async {
    final detail = await transposeSong(
      input:
          '{title: Santo}\n{key: C}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]santo\n',
      toKey: 'D',
    );

    expect(detail.key, 'D');
    expect(
      detail.sections.first.lines.first.chords
          .map((ChordMark mark) => mark.chord)
          .toList(),
      <String>['D', 'A'],
    );
  });

  test('key_choices devolve os doze tons', () async {
    final tons = await keyChoices();
    expect(tons, hasLength(12));
    expect(tons, contains('C'));
    expect(tons, contains('Bb'));
  });

  // ── Agenda e biblioteca: o app lendo o que ele mesmo gravou ─────────────
  //
  // Antes desta correção, `salvar_na_biblioteca` deixava só `<id>/meta.json` e
  // `<id>/song.cho`: a leitura seguinte respondia
  // "missing file: .../library.json" e nem a biblioteca nem a agenda listavam
  // nada. Estes testes usam o cdylib real e o sistema de arquivos real.

  Directory diretorioDeTeste() {
    final dir = Directory.systemTemp.createTempSync('kentron-bridge-');
    addTearDown(() {
      if (dir.existsSync()) dir.deleteSync(recursive: true);
    });
    return dir;
  }

  const cifraCurta = '{title: Santo}\n{key: C}\n\n[Verse]\n[C]texto\n';

  test('a biblioteca que o app salva fica listável e removível', () async {
    final dir = diretorioDeTeste();

    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);
    await salvarNaBiblioteca(
      baseDir: dir.path,
      id: 'hino-3',
      chordpro: '{title: Hino 3}\n{key: G}\n\n[Verse]\n[G]texto\n',
    );

    final salvas = await listarBiblioteca(baseDir: dir.path);
    expect(salvas.map((SongSummary s) => s.title), <String>['Hino 3', 'Santo']);

    await removerDaBiblioteca(baseDir: dir.path, id: 'hino-3');
    final restantes = await listarBiblioteca(baseDir: dir.path);
    expect(restantes.map((SongSummary s) => s.title), <String>['Santo']);
  });

  test('listar_agenda atravessa a ponte e devolve o evento do disco', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);

    // A agenda ainda não tem função de escrita na ponte (fica para o próximo
    // incremento): o teste grava o arquivo no formato que o núcleo lê
    // (`schema_version` 2, execução = obra + versão + tom).
    final setlists = Directory('${dir.path}/setlists')..createSync();
    File('${setlists.path}/culto-1.json').writeAsStringSync(
      '{"schema_version":2,"id":"culto-1","name":"Culto da manhã",'
      '"data":"2026-11-22","tipo":"culto",'
      '"entries":[{"song":"santo","versao":"versão do coral","tom":"D"}]}',
    );

    final eventos = await listarAgenda(baseDir: dir.path);

    expect(eventos, hasLength(1));
    final culto = eventos.single;
    expect(culto.id, 'culto-1');
    expect(culto.nome, 'Culto da manhã');
    expect(culto.data, '2026-11-22');
    expect(culto.tipo, 'culto');
    expect(culto.execucoes, hasLength(1));
    expect(culto.execucoes.single.songId, 'santo');
    expect(culto.execucoes.single.titulo, 'Santo');
    expect(culto.execucoes.single.versao, 'versão do coral');
    expect(culto.execucoes.single.tom, 'D');
    expect(culto.core, startsWith('kentron_ffi '));
  });

  test('listar_agenda em pasta inexistente vira exceção, não lista vazia', () async {
    final dir = diretorioDeTeste();

    await expectLater(
      listarAgenda(baseDir: '${dir.path}/nao-existe'),
      throwsA(predicate((Object erro) => '$erro'.contains('não existe'))),
    );
  });

  test('biblioteca sem agenda devolve zero eventos, e não erro', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);

    expect(await listarAgenda(baseDir: dir.path), isEmpty);
  });

  test('remover obra usada por um evento é recusado pela ponte', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);
    final setlists = Directory('${dir.path}/setlists')..createSync();
    File('${setlists.path}/culto-1.json').writeAsStringSync(
      '{"schema_version":2,"id":"culto-1","name":"Culto da manhã",'
      '"entries":[{"song":"santo","versao":null,"tom":null}]}',
    );

    await expectLater(
      removerDaBiblioteca(baseDir: dir.path, id: 'santo'),
      throwsA(predicate((Object erro) => '$erro'.contains('Culto da manhã'))),
    );
    // A obra e o evento continuam legíveis.
    expect(await listarAgenda(baseDir: dir.path), hasLength(1));
  });

  // ── Escrever a agenda pela ponte ────────────────────────────────────────
  //
  // O caminho inteiro: a função da ponte grava pelo `notation_bundle` no sistema
  // de arquivos real, e a leitura seguinte é a mesma que a tela usa.

  test('listar biblioteca carrega o id da obra, não só o título', () async {
    final dir = diretorioDeTeste();
    // O id NÃO é o slug do título: é o que a agenda grava.
    await salvarNaBiblioteca(baseDir: dir.path, id: 'hino-3', chordpro: cifraCurta);

    final musicas = await listarBiblioteca(baseDir: dir.path);
    expect(musicas.single.id, 'hino-3');
    expect(musicas.single.title, 'Santo');
  });

  test('a agenda que o app grava é a agenda que ele lê', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);
    await salvarNaBiblioteca(
      baseDir: dir.path,
      id: 'hino-3',
      chordpro: '{title: Hino 3}\n{key: G}\n\n[Verse]\n[G]texto\n',
    );

    final gravado = await salvarEventoAgenda(
      baseDir: dir.path,
      id: 'culto-1',
      nome: 'Culto da manhã',
      data: '2026-11-22',
      tipo: 'culto',
      execucoes: const <EntradaAgenda>[
        EntradaAgenda(songId: 'santo', versao: 'versão do coral', tom: 'D'),
        EntradaAgenda(songId: 'hino-3'),
      ],
    );

    expect(gravado.id, 'culto-1');
    expect(gravado.core, startsWith('kentron_ffi '));
    expect(gravado.execucoes, hasLength(2));

    final eventos = await listarAgenda(baseDir: dir.path);
    expect(eventos, hasLength(1));
    final culto = eventos.single;
    expect(culto.nome, 'Culto da manhã');
    expect(culto.data, '2026-11-22');
    expect(culto.tipo, 'culto');
    expect(culto.execucoes.map((ExecucaoAgenda e) => e.songId), <String>[
      'santo',
      'hino-3',
    ]);
    expect(culto.execucoes.first.titulo, 'Santo', reason: 'o título vem do bundle');
    expect(culto.execucoes.first.versao, 'versão do coral');
    expect(culto.execucoes.first.tom, 'D');
    expect(
      culto.execucoes.last.versao,
      isNull,
      reason: 'versão não declarada não é inventada',
    );
  });

  test('gravar evento com obra fora da biblioteca vira exceção', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);

    await expectLater(
      salvarEventoAgenda(
        baseDir: dir.path,
        id: 'culto-1',
        nome: 'Culto da manhã',
        execucoes: const <EntradaAgenda>[EntradaAgenda(songId: 'fantasma')],
      ),
      throwsA(predicate((Object erro) => '$erro'.contains('fantasma'))),
    );
    // Nada foi gravado: a leitura seguinte continua sendo "sem agenda".
    expect(await listarAgenda(baseDir: dir.path), isEmpty);
  });

  // ── Repetir a agenda anterior ───────────────────────────────────────────

  test('repetir a agenda anterior traz o tom atual da biblioteca', () async {
    final dir = diretorioDeTeste();
    // A cifra está em `E` hoje; o evento anterior declarou `D`.
    await salvarNaBiblioteca(
      baseDir: dir.path,
      id: 'santo',
      chordpro: '{title: Santo}\n{key: E}\n\n[Verse]\n[E]texto\n',
    );
    await salvarEventoAgenda(
      baseDir: dir.path,
      id: 'culto-1',
      nome: 'Culto da manhã',
      data: '2026-11-22',
      tipo: 'culto',
      execucoes: const <EntradaAgenda>[
        EntradaAgenda(songId: 'santo', versao: 'versão do coral', tom: 'D'),
      ],
    );

    final novo = await repetirUltimoEvento(
      baseDir: dir.path,
      novoId: 'culto-2',
      nome: 'Culto da noite',
      data: '2026-11-29',
    );

    expect(novo.id, 'culto-2');
    expect(novo.tipo, 'culto', reason: 'o tipo vem do evento anterior');
    expect(novo.execucoes.single.songId, 'santo');
    expect(novo.execucoes.single.tom, 'E', reason: 'o tom é o da biblioteca agora');
    expect(novo.execucoes.single.versao, 'versão do coral');

    // Os dois eventos estão no disco, e o anterior segue com o tom dele.
    final eventos = await listarAgenda(baseDir: dir.path);
    expect(eventos.map((EventoAgenda e) => e.id), <String>['culto-1', 'culto-2']);
    expect(eventos.first.execucoes.single.tom, 'D');
  });

  test('repetir sem evento anterior vira exceção, não evento vazio', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);

    await expectLater(
      repetirUltimoEvento(
        baseDir: dir.path,
        novoId: 'culto-2',
        nome: 'Culto da noite',
      ),
      throwsA(predicate((Object erro) => '$erro'.contains('não há evento anterior'))),
    );
    expect(await listarAgenda(baseDir: dir.path), isEmpty);
  });

  // ── O relatório de uso na ponte ─────────────────────────────────────────

  test('o relatório de uso chega com o uso e com as obras sem ficha', () async {
    final dir = diretorioDeTeste();
    await salvarNaBiblioteca(baseDir: dir.path, id: 'santo', chordpro: cifraCurta);
    await salvarNaBiblioteca(
      baseDir: dir.path,
      id: 'hino-3',
      chordpro: '{title: Hino 3}\n{key: G}\n\n[Verse]\n[G]texto\n',
    );
    // Terceira obra, sem ficha: com 1 pública e 1 sem ficha os dois números
    // coincidiriam e o teste não separaria um do outro.
    await salvarNaBiblioteca(
      baseDir: dir.path,
      id: 'sem-ficha',
      chordpro: '{title: Obra Sem Ficha}\n{key: G}\n\n[Verse]\n[G]texto\n',
    );
    await salvarEventoAgenda(
      baseDir: dir.path,
      id: 'culto-1',
      nome: 'Culto da manhã',
      data: '2026-11-22',
      tipo: 'culto',
      execucoes: const <EntradaAgenda>[
        EntradaAgenda(songId: 'santo'),
        EntradaAgenda(songId: 'hino-3'),
      ],
    );
    await salvarEventoAgenda(
      baseDir: dir.path,
      id: 'culto-2',
      nome: 'Culto da noite',
      data: '2026-11-29',
      tipo: 'culto',
      execucoes: const <EntradaAgenda>[
        EntradaAgenda(songId: 'santo', versao: 'versão do coral'),
        EntradaAgenda(songId: 'sem-ficha'),
      ],
    );

    final relatorio = await relatorioDeUso(baseDir: dir.path, anoCorrente: 2026);

    expect(relatorio.eventos, 2);
    expect(relatorio.execucoes, 4);
    expect(relatorio.obrasDistintas, 3);
    expect(relatorio.core, startsWith('kentron_ffi '));
    expect(
      relatorio.obras.map((ObraDoRelatorio o) => o.titulo),
      <String>['Santo', 'Hino 3', 'Obra Sem Ficha'],
      reason: 'mais cantada primeiro, depois por título',
    );
    final santo = relatorio.obras.first;
    expect(santo.execucoes, 2);
    expect(santo.publica, isTrue, reason: 'ficha do Cantor Cristão 09');
    expect(santo.veredito, contains('domínio público'));
    expect(relatorio.publicas, 1);
    expect(relatorio.semFicha, 2);
    for (final ObraDoRelatorio obra in relatorio.obras.skip(1)) {
      expect(obra.veredito, isNull, reason: 'sem ficha é sem ficha');
      expect(obra.publica, isFalse);
    }
  });

  test('relatório de uso sem biblioteca vira exceção, não relatório vazio', () async {
    final dir = diretorioDeTeste();

    await expectLater(
      relatorioDeUso(baseDir: dir.path, anoCorrente: 2026),
      throwsA(predicate((Object erro) => '$erro'.contains('library.json'))),
    );
  });
}
