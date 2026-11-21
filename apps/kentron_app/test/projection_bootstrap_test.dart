import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/projection_bootstrap.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

const _cifra = SongDetail(
  title: 'Santo',
  sections: <SongSection>[
    SongSection(
      name: 'Verse',
      lines: <SongLine>[
        SongLine(text: 'Santo, santo', chords: <ChordMark>[]),
      ],
    ),
  ],
  core: 'falso',
);

Future<SongDetail> _ler(String _) async => _cifra;

void main() {
  testWidgets('abre direto na projeção', (tester) async {
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _ler,
        locale: Locale('en'),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ProjectionPage), findsOneWidget);
    expect(find.text('Santo, santo'), findsOneWidget);
  });

  testWidgets('mostra a falha do núcleo em vez de tela vazia', (tester) async {
    Future<SongDetail> quebrado(String _) async {
      throw StateError('ponte caiu');
    }

    await tester.pumpWidget(
      ProjecaoApp(
        chordpro: 'ignorado',
        ler: quebrado,
        locale: const Locale('en'),
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
