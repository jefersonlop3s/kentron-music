import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/rust/frb_generated.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:material_ui/material_ui.dart';

/// Sovereign Emerald, token visual do DESIGN.md.
const _sovereignEmerald = Color(0xFF10B981);

/// Cifra de exemplo da tela inicial: **só acordes, sem letra de terceiros**.
/// O objetivo é provar a ponte (ADR-0007), não entregar o repertório.
const cifraDeExemplo = '''
{title: Exemplo}
{key: C}
{artist: Kentron}

[Verse]
[C] [G] [Am] [F]
[C] [G] [F] [C]

[Chorus]
[F] [C] [G] [Am]
''';

/// Lê a cifra pelo núcleo Rust. É o caminho real; os testes herméticos
/// injetam outro carregador para não depender de FFI.
Future<SongSummary> carregarPeloNucleo(String chordpro) =>
    parseSong(input: chordpro);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // A tela inicial chama o núcleo Rust: sem isto a ponte não está carregada.
  await RustLib.init();
  runApp(const KentronApp());
}

class KentronApp extends StatelessWidget {
  const KentronApp({
    super.key,
    this.locale,
    this.carregar = carregarPeloNucleo,
    this.cifra = cifraDeExemplo,
  });

  /// Quando informado, força o locale. Os testes usam isso para provar a
  /// tradução sem depender do idioma do aparelho; o app em produção deixa
  /// nulo e o sistema decide.
  final Locale? locale;

  /// Carregador da cifra repassado à tela inicial. O default é o núcleo Rust;
  /// os testes de widget injetam outro para não depender de FFI.
  final Future<SongSummary> Function(String chordpro) carregar;

  /// Cifra mostrada na tela inicial.
  final String cifra;

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
      home: HomeScreen(carregar: carregar, cifra: cifra),
    );
  }
}

/// Tela inicial: a saudação vem do `core_l10n` e o conteúdo vem do núcleo Rust.
class HomeScreen extends StatefulWidget {
  const HomeScreen({
    super.key,
    this.carregar = carregarPeloNucleo,
    this.cifra = cifraDeExemplo,
  });

  /// De onde vem o resumo da cifra. O default é o núcleo Rust de verdade; os
  /// testes herméticos injetam outro carregador.
  final Future<SongSummary> Function(String chordpro) carregar;

  /// Cifra mostrada na tela inicial.
  final String cifra;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  late final Future<SongSummary> _resumo = widget.carregar(widget.cifra);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Kentron')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Text(CoreL10n.of(context).homeWelcome),
            const SizedBox(height: 24),
            FutureBuilder<SongSummary>(
              future: _resumo,
              builder: (context, snapshot) {
                if (snapshot.hasError) {
                  // Fail-loud: falha da ponte aparece na tela, não vira vazio.
                  return Text('Núcleo indisponível: ${snapshot.error}');
                }
                final resumo = snapshot.data;
                if (resumo == null) {
                  return const CircularProgressIndicator();
                }
                return _CartaoDaCifra(resumo: resumo);
              },
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => SongReaderPage(input: widget.cifra),
                ),
              ),
              child: Text(CoreL10n.of(context).stageOpenReader),
            ),
          ],
        ),
      ),
    );
  }
}

/// O que o núcleo Rust devolveu, desenhado para o usuário ver a ponte viva.
class _CartaoDaCifra extends StatelessWidget {
  const _CartaoDaCifra({required this.resumo});

  final SongSummary resumo;

  @override
  Widget build(BuildContext context) {
    final estilo = Theme.of(context).textTheme;
    return Column(
      children: <Widget>[
        Text(resumo.title, style: estilo.titleLarge),
        Text(
          '${resumo.artist ?? 'sem artista'} · tom ${resumo.key ?? '?'} · '
          '${resumo.sections} seção(ões)',
          style: estilo.bodyMedium,
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          children: <Widget>[
            for (final acorde in resumo.chords) Chip(label: Text(acorde)),
          ],
        ),
        const SizedBox(height: 12),
        Text('núcleo: ${resumo.core}', style: estilo.labelSmall),
      ],
    );
  }
}
