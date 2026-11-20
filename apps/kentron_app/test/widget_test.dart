import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/main.dart';

void main() {
  testWidgets('renders the Kentron shell', (tester) async {
    await tester.pumpWidget(const KentronApp());

    expect(find.text('Kentron'), findsOneWidget);
    expect(find.text('Welcome to Kentron Music Hub'), findsOneWidget);
  });
}
