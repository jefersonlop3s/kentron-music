import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/chord_line.dart';
import 'package:kentron_app/src/stage/projection_page.dart';
import 'package:material_ui/material_ui.dart';

/// Tokens do DESIGN.md usados no Modo Palco.
const _slateInk = Color(0xFF0B0F14);
const _paperWhite = Color(0xFFF2F5F7);
const _vibrantGold = Color(0xFFE0B341);

/// Tipografia de cifra: Mono, como manda o DESIGN.md (§3).
const _mono = TextStyle(
  fontFamily: 'monospace',
  fontFamilyFallback: <String>['Menlo', 'Courier New', 'Courier'],
  fontSize: 16,
  height: 1.5,
);

/// Lê a cifra pelo núcleo. Existe como função de topo porque a ponte recebe
/// `{required String input}` (nomeado) e o widget precisa de `Function(String)`:
/// o adaptador casa as duas formas sem espalhar parâmetro nomeado na UI.
Future<SongDetail> lerDoNucleo(String input) => readSong(input: input);

/// Transpõe pelo núcleo. Existe como função de topo para o widget poder receber
/// uma versão de mentira nos testes, sem tocar em FFI.
Future<SongDetail> transposeToKey(String input, String toKey) =>
    transposeSong(input: input, toKey: toKey);

/// Modo Palco: a cifra inteira na tela, acorde sobre a letra, transposição por tom.
///
/// Carrega do núcleo Rust por injeção (`ler`), não por chamada direta: o widget
/// fica hermético nos testes e o mesmo código serve ao app real.
class SongReaderPage extends StatefulWidget {
  const SongReaderPage({
    super.key,
    required this.input,
    this.ler = lerDoNucleo,
    this.transpor = transposeToKey,
    this.tons = keyChoices,
  });

  /// A cifra em ChordPro, como veio do arquivo ou do exemplo.
  final String input;

  /// De onde vem a estrutura da cifra. Default: núcleo Rust.
  final Future<SongDetail> Function(String input) ler;

  /// Como transpõe. Default: núcleo Rust.
  final Future<SongDetail> Function(String input, String toKey) transpor;

  /// Os tons oferecidos ao músico. Default: núcleo Rust.
  final Future<List<String>> Function() tons;

  @override
  State<SongReaderPage> createState() => _SongReaderPageState();
}

class _SongReaderPageState extends State<SongReaderPage> {
  SongDetail? _cifra;
  List<String> _tons = const <String>[];
  Object? _erro;
  bool _palco = false;
  bool _trocandoTom = false;

  @override
  void initState() {
    super.initState();
    _carregar();
  }

  Future<void> _carregar() async {
    try {
      final cifra = await widget.ler(widget.input);
      final tons = await widget.tons();
      if (!mounted) return;
      setState(() {
        _cifra = cifra;
        _tons = tons;
        _erro = null;
      });
    } on Object catch (erro) {
      if (!mounted) return;
      setState(() => _erro = erro);
    }
  }

  Future<void> _transporPara(String tom) async {
    if (_trocandoTom) return;
    setState(() => _trocandoTom = true);
    try {
      final cifra = await widget.transpor(widget.input, tom);
      if (!mounted) return;
      setState(() {
        _cifra = cifra;
        _erro = null;
      });
    } on Object catch (erro) {
      if (!mounted) return;
      setState(() => _erro = erro);
    } finally {
      if (mounted) setState(() => _trocandoTom = false);
    }
  }

  void _abrirProjecao() {
    final cifra = _cifra;
    if (cifra == null) return;
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => ProjectionPage(song: cifra),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = CoreL10n.of(context);
    final cifra = _cifra;
    final titulo = (cifra?.title.isNotEmpty ?? false)
        ? cifra!.title
        : l10n.docChordChart;
    return Scaffold(
      backgroundColor: _palco ? _slateInk : null,
      appBar: AppBar(
        title: Text(titulo),
        actions: <Widget>[
          IconButton(
            tooltip: l10n.stageProject,
            onPressed: _cifra == null ? null : _abrirProjecao,
            icon: const Icon(Icons.slideshow),
          ),
          IconButton(
            tooltip: l10n.stageContrast,
            onPressed: () => setState(() => _palco = !_palco),
            icon: Icon(_palco ? Icons.light_mode : Icons.dark_mode),
          ),
        ],
      ),
      body: _erro != null ? _ErroDoNucleo(erro: _erro!) : _corpo(cifra),
    );
  }

  Widget _corpo(SongDetail? cifra) {
    final l10n = CoreL10n.of(context);
    if (cifra == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final cor = _palco ? _paperWhite : _slateInk;
    final tom = cifra.key;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
          child: Row(
            children: <Widget>[
              Text('${l10n.stageKey}: ', style: _mono.copyWith(color: cor)),
              DropdownButton<String>(
                value: _tons.contains(tom) ? tom : null,
                hint: Text(tom ?? '—', style: _mono.copyWith(color: cor)),
                items: <DropdownMenuItem<String>>[
                  for (final t in _tons)
                    DropdownMenuItem<String>(value: t, child: Text(t)),
                ],
                onChanged: _trocandoTom
                    ? null
                    : (t) {
                        if (t != null) _transporPara(t);
                      },
              ),
            ],
          ),
        ),
        if (_trocandoTom) const LinearProgressIndicator(minHeight: 2),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                for (final secao in cifra.sections)
                  _SecaoDaCifra(secao: secao, cor: cor),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// Uma seção: o nome entre colchetes e as linhas com acorde sobre a letra.
class _SecaoDaCifra extends StatelessWidget {
  const _SecaoDaCifra({required this.secao, required this.cor});

  final SongSection secao;
  final Color cor;

  @override
  Widget build(BuildContext context) {
    final nome = secao.name;
    return Padding(
      padding: const EdgeInsets.only(bottom: 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (nome != null && nome.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: Text(
                nome,
                style: _mono.copyWith(
                  color: _vibrantGold,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          for (final linha in secao.lines) _LinhaDaCifra(linha: linha, cor: cor),
        ],
      ),
    );
  }
}

/// Uma linha: acordes na linha de cima, letra embaixo.
class _LinhaDaCifra extends StatelessWidget {
  const _LinhaDaCifra({required this.linha, required this.cor});

  final SongLine linha;
  final Color cor;

  @override
  Widget build(BuildContext context) {
    final acordes = chordLine(linha.chords);
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (acordes.isNotEmpty)
            Text(acordes, style: _mono.copyWith(color: _vibrantGold)),
          if (linha.text.isNotEmpty)
            Text(linha.text, style: _mono.copyWith(color: cor)),
        ],
      ),
    );
  }
}

/// Falha do núcleo: mostra o erro, não engole.
class _ErroDoNucleo extends StatelessWidget {
  const _ErroDoNucleo({required this.erro});

  final Object erro;

  @override
  Widget build(BuildContext context) {
    final l10n = CoreL10n.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            Icon(
              Icons.error_outline,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(height: 12),
            Text(
              '${l10n.stageLoadFailed}: $erro',
              textAlign: TextAlign.center,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ],
        ),
      ),
    );
  }
}
