import 'package:core_l10n/core_l10n.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

/// Cifra de 2 seções, 1 linha cada — sem paginação intra-seção.
const _cifra = SongDetail(
  title: 'Santo',
  key: 'C',
  sections: <SongSection>[
    SongSection(
      name: 'Verse',
      lines: <SongLine>[
        SongLine(
          text: 'Santo, santo',
          chords: <ChordMark>[ChordMark(position: 0, chord: 'C')],
        ),
      ],
    ),
    SongSection(
      name: 'Chorus',
      lines: <SongLine>[
        SongLine(
          text: 'Cantarei',
          chords: <ChordMark>[ChordMark(position: 0, chord: 'G')],
        ),
      ],
    ),
  ],
  core: 'falso 0.0.0',
);

/// Cifra com seção longa (10 linhas) que exige paginação.
const _cifraLonga = SongDetail(
  title: 'Longa',
  key: 'C',
  sections: <SongSection>[
    SongSection(
      name: 'Verse',
      lines: <SongLine>[
        SongLine(text: 'Linha 01', chords: <ChordMark>[]),
        SongLine(text: 'Linha 02', chords: <ChordMark>[]),
        SongLine(text: 'Linha 03', chords: <ChordMark>[]),
        SongLine(text: 'Linha 04', chords: <ChordMark>[]),
        SongLine(text: 'Linha 05', chords: <ChordMark>[]),
        SongLine(text: 'Linha 06', chords: <ChordMark>[]),
        SongLine(text: 'Linha 07', chords: <ChordMark>[]),
        SongLine(text: 'Linha 08', chords: <ChordMark>[]),
        SongLine(text: 'Linha 09', chords: <ChordMark>[]),
        SongLine(text: 'Linha 10', chords: <ChordMark>[]),
      ],
    ),
    SongSection(
      name: 'Chorus',
      lines: <SongLine>[
        SongLine(text: 'Refrão', chords: <ChordMark>[]),
      ],
    ),
  ],
  core: 'falso 0.0.0',
);

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
  testWidgets('mostra a primeira seção sem acorde', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifra)));

    expect(find.text('Verse'), findsOneWidget);
    expect(find.text('Santo, santo'), findsOneWidget);
    expect(find.text('1 / 2'), findsOneWidget);
    // Projeção é para a congregação: acorde não aparece.
    expect(find.text('C'), findsNothing);
  });

  testWidgets('avança e volta por seção', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifra)));

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();
    expect(find.text('Chorus'), findsOneWidget);
    expect(find.text('2 / 2'), findsOneWidget);

    await tester.tap(find.byTooltip('Previous section'));
    await tester.pumpAndSettle();
    expect(find.text('Verse'), findsOneWidget);
    expect(find.text('1 / 2'), findsOneWidget);
  });

  testWidgets('desabilita o avanço nos limites', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifra)));

    final anterior = tester.widget<IconButton>(
      find.widgetWithIcon(IconButton, Icons.chevron_left),
    );
    expect(anterior.onPressed, isNull, reason: 'primeira seção não tem anterior');

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();

    final proxima = tester.widget<IconButton>(
      find.widgetWithIcon(IconButton, Icons.chevron_right),
    );
    expect(proxima.onPressed, isNull, reason: 'última seção não tem próxima');
  });

  testWidgets('toque na metade direita avança', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifra)));

    await tester.tapAt(const Offset(700, 300));
    await tester.pumpAndSettle();
    expect(find.text('Chorus'), findsOneWidget);
  });

  testWidgets('seta do teclado avança', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifra)));

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pumpAndSettle();
    expect(find.text('Chorus'), findsOneWidget);
  });

  // ── Paginação intra-seção ─────────────────────────────────────────────

  testWidgets('seção longa divide em sub-páginas', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifraLonga)));

    expect(find.text('Linha 01'), findsOneWidget);
    expect(find.text('Linha 06'), findsOneWidget);
    expect(find.text('Linha 07'), findsNothing);
    expect(find.text('Linha 10'), findsNothing);
    expect(find.text('1 / 2 · 1 / 2'), findsOneWidget);
  });

  testWidgets('avança sub-página dentro da mesma seção', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifraLonga)));

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();

    expect(find.text('Linha 07'), findsOneWidget);
    expect(find.text('Linha 10'), findsOneWidget);
    expect(find.text('Linha 01'), findsNothing);
    expect(find.text('1 / 2 · 2 / 2'), findsOneWidget);
  });

  testWidgets('volta sub-página dentro da mesma seção', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifraLonga)));

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Previous section'));
    await tester.pumpAndSettle();

    expect(find.text('Linha 01'), findsOneWidget);
    expect(find.text('1 / 2 · 1 / 2'), findsOneWidget);
  });

  testWidgets('avança para próxima seção após última sub-página', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifraLonga)));

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();

    expect(find.text('Refrão'), findsOneWidget);
    expect(find.text('2 / 2'), findsOneWidget);
  });

  testWidgets('volta para última sub-página da seção anterior', (tester) async {
    await tester.pumpWidget(_montar(const ProjectionPage(song: _cifraLonga)));

    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Next section'));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Previous section'));
    await tester.pumpAndSettle();

    expect(find.text('Linha 07'), findsOneWidget);
    expect(find.text('1 / 2 · 2 / 2'), findsOneWidget);
  });

  // ── Pseudolocalização: textos longos não quebram ──────────────────────

  testWidgets('seção com texto muito longo não quebra layout', (tester) async {

    const longa = SongDetail(
      title: 'Pseudoloc',
      key: 'C',
      sections: <SongSection>[
        SongSection(
          name: 'Longa seção com nome extenso para testar layout',
          lines: <SongLine>[
            SongLine(
              text: 'Texto extremamente longo que deve caber na projeção sem quebrar o layout ou causar overflow — esta é a pseudolocalização em ação para garantir que a UI suporta textos expandidos.',
              chords: <ChordMark>[],
            ),
          ],
        ),
      ],
      core: 'falso',
    );

    await tester.pumpWidget(_montar(const ProjectionPage(song: longa)));

    expect(find.textContaining('pseudolocalização'), findsOneWidget);
    expect(find.textContaining('quebrar'), findsOneWidget);
  });

  testWidgets('acentos na projeção são renderizados', (tester) async {

    const acentuada = SongDetail(
      title: 'Acentos',
      key: 'C',
      sections: <SongSection>[
        SongSection(
          name: 'Refrão',
          lines: <SongLine>[
            SongLine(text: 'Coração alegre canta', chords: <ChordMark>[]),
            SongLine(text: 'Louvor ao grande Rei', chords: <ChordMark>[]),
            SongLine(text: 'Alegría del Señor', chords: <ChordMark>[]),
            SongLine(text: 'Seine Güte währet ewiglich', chords: <ChordMark>[]),
          ],
        ),
      ],
      core: 'falso',
    );

    await tester.pumpWidget(_montar(const ProjectionPage(song: acentuada)));

    expect(find.text('Coração alegre canta'), findsOneWidget);
    expect(find.text('Alegría del Señor'), findsOneWidget);
    expect(find.text('Seine Güte währet ewiglich'), findsOneWidget);
  });
}
