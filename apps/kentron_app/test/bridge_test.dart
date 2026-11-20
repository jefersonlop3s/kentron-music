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
}
