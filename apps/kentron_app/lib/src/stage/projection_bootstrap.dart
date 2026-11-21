import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

/// Sovereign Emerald, token do DESIGN.md.
const _sovereignEmerald = Color(0xFF10B981);

/// Tokens de projeção do DESIGN.md.
const _slateInk = Color(0xFF0B0F14);
const _paperWhite = Color(0xFFF2F5F7);

/// Lê a cifra pelo núcleo. Adaptador de topo: a ponte recebe parâmetro nomeado.
Future<SongDetail> lerParaProjecao(String input) => readSong(input: input);

/// Aplicativo mínimo do Browser Source: sem tela inicial, sem navegação.
///
/// Recebe a cifra decodificada da URL e lê pelo núcleo. O carregador é
/// injetável para o teste rodar sem FFI; em produção é o núcleo de verdade.
class ProjecaoApp extends StatelessWidget {
  const ProjecaoApp({
    super.key,
    required this.chordpro,
    this.ler = lerParaProjecao,
    this.locale,
  });

  /// A cifra em ChordPro, como veio da URL.
  final String chordpro;

  /// De onde vem a estrutura. Default: núcleo Rust.
  final Future<SongDetail> Function(String input) ler;

  /// Força o locale; em produção fica nulo e o sistema decide.
  final Locale? locale;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Kentron · Projeção',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(seedColor: _sovereignEmerald),
      ),
      locale: locale,
      localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
        CoreL10n.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: CoreL10n.supportedLocales,
      home: _ProjecaoDoNucleo(chordpro: chordpro, ler: ler),
    );
  }
}

/// Lê a cifra e entrega a projeção; falha aparece, não vira tela vazia.
class _ProjecaoDoNucleo extends StatefulWidget {
  const _ProjecaoDoNucleo({required this.chordpro, required this.ler});

  final String chordpro;
  final Future<SongDetail> Function(String input) ler;

  @override
  State<_ProjecaoDoNucleo> createState() => _ProjecaoDoNucleoState();
}

class _ProjecaoDoNucleoState extends State<_ProjecaoDoNucleo> {
  late final Future<SongDetail> _cifra = widget.ler(widget.chordpro);

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<SongDetail>(
      future: _cifra,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return Scaffold(
            backgroundColor: _slateInk,
            body: Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Text(
                  '${CoreL10n.of(context).stageLoadFailed}: ${snapshot.error}',
                  textAlign: TextAlign.center,
                  style: const TextStyle(color: _paperWhite),
                ),
              ),
            ),
          );
        }
        final cifra = snapshot.data;
        if (cifra == null) {
          return const Scaffold(
            backgroundColor: _slateInk,
            body: Center(child: CircularProgressIndicator()),
          );
        }
        return ProjectionPage(song: cifra);
      },
    );
  }
}
