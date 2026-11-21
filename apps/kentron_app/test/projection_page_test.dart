import 'package:core_l10n/core_l10n.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

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
}
