import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/main.dart';
import 'package:material_ui/material_ui.dart';

void main() {
  testWidgets('renders the Kentron shell', (tester) async {
    await tester.pumpWidget(const KentronApp(locale: Locale('en')));

    expect(find.text('Kentron'), findsOneWidget);
  });

  testWidgets('greeting comes from core_l10n, not from the widget', (
    tester,
  ) async {
    await tester.pumpWidget(const KentronApp(locale: Locale('en')));
    expect(find.text('Welcome to Kentron Music Hub'), findsOneWidget);

    await tester.pumpWidget(const KentronApp(locale: Locale('pt')));
    expect(find.text('Bem-vindo ao Kentron Music Hub'), findsOneWidget);

    await tester.pumpWidget(const KentronApp(locale: Locale('es')));
    expect(find.text('Bienvenido a Kentron Music Hub'), findsOneWidget);
  });

  testWidgets('regional variant falls back to its base language', (
    tester,
  ) async {
    await tester.pumpWidget(const KentronApp(locale: Locale('pt', 'PT')));
    expect(find.text('Bem-vindo ao Kentron Music Hub'), findsOneWidget);
  });
}
