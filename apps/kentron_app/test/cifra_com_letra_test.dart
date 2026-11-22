//! A letra na cifra, provada ponta a ponta: o motor gera, o app desenha.
//!
//! A cifra vem do **arquivo de fixture gerado pelo `mir_engine`**
//! (`crates/mir_engine/tests/fixtures/cifra-com-letra.cho`), que o teste dourado
//! de Rust confere contra o gerador. Uma fonte só: se o motor mudar o formato, os
//! dois lados quebram juntos, em vez de um passar e o outro mentir.
//!
//! A ponte é a **real** (`readSong` sobre o `cdylib`), não uma função de mentira:
//! a pergunta é se o app desenha o que o motor produz. Exige o cdylib:
//!
//! ```text
//! cargo build -p kentron_ffi
//! ```

import 'dart:io';

import 'package:core_l10n/core_l10n.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/rust/frb_generated.dart';
import 'package:kentron_app/src/stage/chord_line.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:material_ui/material_ui.dart';

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

/// A cifra gerada pelo motor de análise (ADR-0012).
String _cifraDoMotor() {
  final arquivo = File(
    '../../crates/mir_engine/tests/fixtures/cifra-com-letra.cho',
  );
  if (!arquivo.existsSync()) {
    fail(
      'fixture do motor ausente em ${arquivo.absolute.path}. '
      'Gere com: MIR_ATUALIZAR_FIXTURE=1 cargo test -p mir_engine fixture',
    );
  }
  return arquivo.readAsStringSync();
}

Widget _montar(Widget filho) => MaterialApp(
  locale: const Locale('en'),
  localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
    CoreL10n.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: CoreL10n.supportedLocales,
  home: filho,
);

void main() {
  setUpAll(() async {
    await RustLib.init(externalLibrary: ExternalLibrary.open(_libraryPath()));
  });

  test('o núcleo devolve a letra do motor com os acordes por cima dela', () async {
    final musica = await readSong(input: _cifraDoMotor());

    expect(musica.title, 'Amazing Grace');
    // Procedência viaja na cifra: quem lê sabe de onde a letra veio.
    expect(musica.core, startsWith('kentron_ffi '));

    final linhas = musica.sections.expand((s) => s.lines).toList();
    final comLetra = linhas.where((l) => l.text.isNotEmpty).toList();
    expect(comLetra.length, 4, reason: 'a estrofe tem quatro linhas');
    expect(comLetra.first.text, 'Amazing grace! how sweet the sound');
    expect(comLetra[1].text, 'That saved a wretch like me!');
    // Cada linha com letra tem acorde — e o acorde cai dentro do texto.
    for (final linha in comLetra) {
      expect(linha.chords, isNotEmpty, reason: 'linha sem acorde: ${linha.text}');
      for (final acorde in linha.chords) {
        expect(
          acorde.position,
          lessThanOrEqualTo(linha.text.length),
          reason: 'acorde fora da linha: ${acorde.chord} em "${linha.text}"',
        );
      }
    }
    // O primeiro acorde da primeira linha começa no começo dela.
    expect(comLetra.first.chords.first.position, 0);
  });

  testWidgets('a tela do Modo Palco desenha a letra e o acorde sobre ela', (
    tester,
  ) async {
    // A ponte é chamada dentro de `runAsync`: I/O real não avança na zona de
    // tempo falso do `testWidgets`, e `pumpAndSettle` giraria para sempre
    // esperando uma Future que nunca completa. O desenho é conferido com o que
    // a **ponte de verdade** devolveu — a cadeia inteira é exercitada.
    late SongDetail musica;
    late List<String> tons;
    await tester.runAsync(() async {
      musica = await readSong(input: _cifraDoMotor());
      tons = await keyChoices();
    });

    final linha = musica.sections
        .expand((s) => s.lines)
        .firstWhere((l) => l.text.contains('Amazing grace'));
    final linhaDeAcordes = chordLine(linha.chords);
    expect(linhaDeAcordes.trim(), isNotEmpty);
    // Os quatro blocos cantados da fixture são C, G, Am e F (o teste dourado do
    // Rust fixa o arquivo): a cifra tem de trazer os dois extremos da progressão.
    final acordes = musica.sections
        .expand((s) => s.lines)
        .expand((l) => l.chords)
        .map((c) => c.chord)
        .toSet();
    expect(acordes, containsAll(<String>['C', 'F']), reason: '$acordes');

    // Os três pontos de I/O da tela recebem o que a ponte real devolveu: sem
    // isso o `keyChoices` do `initState` ficaria pendente na zona de tempo falso.
    Future<SongDetail> lerDoNucleo(String _) async => musica;
    Future<List<String>> tonsDoNucleo() async => tons;
    await tester.pumpWidget(
      _montar(
        SongReaderPage(
          input: 'da fixture',
          ler: lerDoNucleo,
          tons: tonsDoNucleo,
        ),
      ),
    );
    await tester.pumpAndSettle();

    // A letra está na tela, inteira e legível.
    expect(find.text('Amazing grace! how sweet the sound'), findsOneWidget);
    expect(find.text('Was blind, but now I see.'), findsOneWidget);
    // E o acorde está **sobre** ela, na fonte monoespaçada que faz a coluna
    // coincidir com a letra — é o que o `chordLine` promete. O finder é por
    // estilo, não por texto solto: com `{key: C}` o chip do tom no AppBar
    // também mostra "C", e `find.text` acharia dois.
    expect(
      find.byWidgetPredicate(
        (widget) =>
            widget is Text &&
            widget.data == linhaDeAcordes &&
            widget.style?.fontFamily == 'monospace',
      ),
      findsOneWidget,
    );
    // A seção carrega o papel inferido do áudio — nuance, não decoração.
    expect(find.textContaining('introducao'), findsWidgets);
  });
}
