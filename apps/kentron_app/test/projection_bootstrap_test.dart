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

/// Duas seções: a projeção abre na que `?secao=N` mandar.
const _duasSecoes = SongDetail(
  title: 'Santo',
  sections: <SongSection>[
    SongSection(
      name: 'Verse',
      lines: <SongLine>[
        SongLine(text: 'Santo, santo', chords: <ChordMark>[]),
      ],
    ),
    SongSection(
      name: 'Chorus',
      lines: <SongLine>[
        SongLine(text: 'Santo é o Senhor', chords: <ChordMark>[]),
      ],
    ),
  ],
  core: 'falso',
);

Future<SongDetail> _lerDuas(String _) async => _duasSecoes;

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

  testWidgets('abre na seção pedida pela URL', (tester) async {
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _lerDuas,
        locale: Locale('en'),
        initialSection: 1,
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Santo é o Senhor'), findsOneWidget);
    expect(find.text('Santo, santo'), findsNothing);
  });

  testWidgets('seção fora da faixa cai na última, sem quebrar', (tester) async {
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _lerDuas,
        locale: Locale('en'),
        initialSection: 99,
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Santo é o Senhor'), findsOneWidget);
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

  // ── Segunda tela: duas projeções independentes mostram o mesmo ────────

  testWidgets('segunda tela mostra o mesmo conteúdo da projeção principal', (
    tester,
  ) async {
    // Simula duas instâncias do ProjecaoApp (operador + segunda tela).
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _lerDuas,
        locale: Locale('en'),
        initialSection: 0,
      ),
    );
    await tester.pumpAndSettle();

    // Operador vê a primeira seção
    expect(find.text('Santo, santo'), findsOneWidget);
    expect(find.byType(ProjectionPage), findsOneWidget);

    // Segunda tela com a mesma cifra, mesma seção — mesmo conteúdo
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _lerDuas,
        locale: Locale('en'),
        initialSection: 0,
      ),
    );
    await tester.pumpAndSettle();

    // Ambas mostram o mesmo
    expect(find.text('Santo, santo'), findsOneWidget);
    expect(find.text('Santo é o Senhor'), findsNothing);
  });

  testWidgets('segunda tela mostra a seção pedida, independente da principal', (
    tester,
  ) async {
    // A segunda tela (TV/projetor) abre com sua própria seção via URL.
    // O conteúdo independe do que o operador vê no app principal.
    await tester.pumpWidget(
      const ProjecaoApp(
        chordpro: 'ignorado',
        ler: _lerDuas,
        locale: Locale('en'),
        initialSection: 1,
      ),
    );
    // Duas pumps: uma para o build, outra para o FutureBuilder resolver.
    await tester.pump();
    await tester.pump();
    expect(find.text('Santo é o Senhor'), findsOneWidget);
    expect(find.text('Santo, santo'), findsNothing);
  });
}
