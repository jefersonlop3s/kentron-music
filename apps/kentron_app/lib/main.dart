import 'dart:io';

import 'package:core_l10n/core_l10n.dart';
import 'package:flutter/foundation.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/rust/frb_generated.dart';
import 'package:kentron_app/src/stage/agenda_page.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:kentron_app/src/stage/projection_bootstrap.dart';
import 'package:kentron_app/src/stage/projection_link.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:material_ui/material_ui.dart';

/// Sovereign Emerald, token visual do DESIGN.md.
const _sovereignEmerald = Color(0xFF10B981);

/// Diretório da biblioteca local (mesma lógica de library_page.dart).
String _bibliotecaDir() {
  if (kIsWeb) return '/tmp/kentron-library';
  return '${Platform.environment['HOME'] ?? '/tmp'}/.kentron/library';
}

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
  await RustLib.init();

  // Browser Source do OBS: URL com a cifra abre direto na projeção.
  final chordpro = chordproDaUrl(Uri.base);
  if (chordpro != null) {
    runApp(
      ProjecaoApp(
        chordpro: chordpro,
        initialSection: secaoDaUrl(Uri.base) ?? 0,
      ),
    );
    return;
  }

  // Restauração: se há cifras na biblioteca, abre na LibraryPage.
  // Senão, mostra a HomeScreen com a cifra de exemplo.
  final Widget telaInicial = await _telaInicialComRestauracao();
  runApp(KentronApp(home: telaInicial));
}

/// Decide a tela inicial: LibraryPage (se há dados) ou HomeScreen (vazio).
Future<Widget> _telaInicialComRestauracao() async {
  try {
    final musicas = await listarBiblioteca(baseDir: _bibliotecaDir());
    if (musicas.isNotEmpty) {
      return const LibraryPage();
    }
  } on Object {
    // Falha ao listar: mostra a HomeScreen mesmo.
  }
  return const HomeScreen();
}

class KentronApp extends StatelessWidget {
  const KentronApp({
    super.key,
    this.locale,
    this.carregar = carregarPeloNucleo,
    this.cifra = cifraDeExemplo,
    this.home,
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

  /// Tela inicial personalizada. Quando nula, usa [HomeScreen].
  final Widget? home;

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
      home: home ?? HomeScreen(carregar: carregar, cifra: cifra),
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
            const SizedBox(height: 12),
            OutlinedButton(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const LibraryPage(),
                ),
              ),
              child: const Text('Biblioteca'),
            ),
            const SizedBox(height: 12),
            OutlinedButton(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const AgendaPage(),
                ),
              ),
              child: Text(CoreL10n.of(context).agendaTitle),
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
