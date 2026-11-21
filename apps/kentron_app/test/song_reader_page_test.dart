import 'package:core_l10n/core_l10n.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/chord_line.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:material_ui/material_ui.dart';

/// Cifra de mentira: o teste não toca em FFI, só na lógica de tela.
const _cifra = SongDetail(
  title: 'Santo',
  key: 'C',
  artist: 'Autor',
  sections: <SongSection>[
    SongSection(
      name: 'Verse',
      lines: <SongLine>[
        SongLine(
          text: 'Santo, santo',
          chords: <ChordMark>[
            ChordMark(position: 0, chord: 'C'),
            ChordMark(position: 7, chord: 'G'),
          ],
        ),
      ],
    ),
  ],
  core: 'falso 0.0.0',
);

Future<SongDetail> _ler(String _) async => _cifra;

Future<SongDetail> _transpor(String _, String toKey) async => SongDetail(
  title: _cifra.title,
  key: toKey,
  artist: _cifra.artist,
  sections: _cifra.sections,
  core: _cifra.core,
);

Future<List<String>> _tons() async => const <String>['C', 'D', 'E'];

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
  group('chordLine', () {
    test('coloca o acorde na coluna da letra', () {
      expect(
        chordLine(const <ChordMark>[
          ChordMark(position: 0, chord: 'C'),
          ChordMark(position: 7, chord: 'G'),
        ]),
        'C      G',
      );
    });

    test('empurra acorde que invadiria o anterior', () {
      expect(
        chordLine(const <ChordMark>[
          ChordMark(position: 0, chord: 'C'),
          ChordMark(position: 0, chord: 'G'),
        ]),
        'C G',
      );
    });
  });

  testWidgets('desenha título, seção, acorde e letra', (tester) async {
    await tester.pumpWidget(
      _montar(
        const SongReaderPage(
          input: 'ignorado',
          ler: _ler,
          transpor: _transpor,
          tons: _tons,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Santo'), findsOneWidget);
    expect(find.text('Verse'), findsOneWidget);
    expect(find.text('C      G'), findsOneWidget);
    expect(find.text('Santo, santo'), findsOneWidget);
  });

  testWidgets('transpõe quando o músico troca o tom', (tester) async {
    String? pedido;
    Future<SongDetail> transporEspiao(String _, String toKey) async {
      pedido = toKey;
      return _transpor('', toKey);
    }

    await tester.pumpWidget(
      _montar(
        SongReaderPage(
          input: 'ignorado',
          ler: _ler,
          transpor: transporEspiao,
          tons: _tons,
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byType(DropdownButton<String>));
    await tester.pumpAndSettle();
    await tester.tap(find.text('D').last);
    await tester.pumpAndSettle();

    expect(pedido, 'D');
  });

  testWidgets('mostra a falha do núcleo em vez de engolir', (tester) async {
    Future<SongDetail> lerQuebrado(String _) async {
      throw StateError('ponte caiu');
    }

    await tester.pumpWidget(
      _montar(
        SongReaderPage(
          input: 'ignorado',
          ler: lerQuebrado,
          transpor: _transpor,
          tons: _tons,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(
      find.textContaining('Could not load the chord chart'),
      findsOneWidget,
    );
    expect(find.textContaining('ponte caiu'), findsOneWidget);
  });
}
