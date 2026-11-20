import 'package:core_l10n/core_l10n.dart';
import 'package:material_ui/material_ui.dart';

/// Sovereign Emerald, token visual do DESIGN.md.
const _sovereignEmerald = Color(0xFF10B981);

void main() {
  runApp(const KentronApp());
}

class KentronApp extends StatelessWidget {
  const KentronApp({super.key, this.locale});

  /// Quando informado, força o locale. Os testes usam isso para provar a
  /// tradução sem depender do idioma do aparelho; o app em produção deixa
  /// nulo e o sistema decide.
  final Locale? locale;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Kentron Music Hub',
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(seedColor: _sovereignEmerald),
      ),
      locale: locale,
      // `GlobalMaterialLocalizations.delegates` (plural) já traz Cupertino +
      // Material + Widgets, e é o que o próprio material_ui recomenda. O
      // gerador emite as Global* do flutter_localizations no arquivo de
      // localizações, mas essas são OUTRO tipo — o MaterialApp do material_ui
      // as recusa ("No MaterialLocalizations found"). Medido em 2026-11-20.
      localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
        CoreL10n.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: CoreL10n.supportedLocales,
      home: const HomeScreen(),
    );
  }
}

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Kentron')),
      body: Center(
        child: Text(CoreL10n.of(context).homeWelcome),
      ),
    );
  }
}
