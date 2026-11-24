import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/main.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

/// Carregador de mentira: mantém estes testes **herméticos**, sem FFI.
/// A prova com o núcleo Rust de verdade vive em `bridge_test.dart`.
Future<SongSummary> _carregarFalso(String _) async => const SongSummary(
  title: 'Exemplo',
  key: 'C',
  artist: 'Teste',
  sections: 2,
  chords: <String>['C', 'G'],
  core: 'falso 0.0.0',
);

void main() {
  testWidgets('renders the Kentron shell', (tester) async {
    await tester.pumpWidget(
      const KentronApp(locale: Locale('en'), carregar: _carregarFalso),
    );

    expect(find.text('Kentron'), findsOneWidget);
  });

  testWidgets('greeting comes from core_l10n, not from the widget', (
    tester,
  ) async {
    await tester.pumpWidget(
      const KentronApp(locale: Locale('en'), carregar: _carregarFalso),
    );
    expect(find.text('Welcome to Kentron Music Hub'), findsOneWidget);

    await tester.pumpWidget(
      const KentronApp(locale: Locale('pt'), carregar: _carregarFalso),
    );
    expect(find.text('Bem-vindo ao Kentron Music Hub'), findsOneWidget);

    await tester.pumpWidget(
      const KentronApp(locale: Locale('es'), carregar: _carregarFalso),
    );
    expect(find.text('Bienvenido a Kentron Music Hub'), findsOneWidget);
  });

  testWidgets('regional variant falls back to its base language', (
    tester,
  ) async {
    await tester.pumpWidget(
      const KentronApp(locale: Locale('pt', 'PT'), carregar: _carregarFalso),
    );
    expect(find.text('Bem-vindo ao Kentron Music Hub'), findsOneWidget);
  });

  testWidgets('the card shows what the loader returned', (tester) async {
    await tester.pumpWidget(
      const KentronApp(locale: Locale('en'), carregar: _carregarFalso),
    );
    await tester.pumpAndSettle();

    expect(find.text('Exemplo'), findsOneWidget);
    expect(find.text('Teste · tom C · 2 seção(ões)'), findsOneWidget);
    expect(find.text('C'), findsOneWidget);
    expect(find.text('G'), findsOneWidget);
    expect(find.text('núcleo: falso 0.0.0'), findsOneWidget);
  });

  testWidgets('a bridge failure appears on screen instead of empty', (
    tester,
  ) async {
    await tester.pumpWidget(
      KentronApp(
        locale: const Locale('en'),
        carregar: (String _) async => throw StateError('ponte fora'),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.textContaining('Núcleo indisponível'), findsOneWidget);
    expect(find.textContaining('ponte fora'), findsOneWidget);
  });

  // ── Alpha assistido: fluxo completo home → modo palco → projeção ─────

  testWidgets('home tem botões de navegação', (tester) async {
    await tester.pumpWidget(
      const KentronApp(locale: Locale('en'), carregar: _carregarFalso),
    );

    expect(find.text('Open chord chart'), findsOneWidget);
    expect(find.text('Biblioteca'), findsOneWidget);
  });
}
