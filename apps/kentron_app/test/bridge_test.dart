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
}
