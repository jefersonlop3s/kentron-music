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
}
