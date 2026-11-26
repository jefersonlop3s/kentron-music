import 'package:core_l10n/core_l10n.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/chord_line.dart';
import 'package:kentron_app/src/stage/projection_link.dart';
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

/// Portão de publicação do motor, exposto à tela. Existe como função de topo
/// para o widget poder receber uma versão de mentira nos testes, sem tocar em FFI.
Future<void> conferirPublicacao(String cifra) => podePublicar(input: cifra);

/// Modo Palco: a cifra inteira na tela, acorde sobre a letra, transposição por tom.
///
/// Carrega do núcleo Rust por injeção (`ler`), não por chamada direta: o widget
/// fica hermético nos testes e o mesmo código serve ao app real.
///
/// Quando [biblioteca] é fornecida, o leitor exibe botões de navegação
/// (anterior/próxima) na AppBar para trocar de música sem voltar à lista.
class SongReaderPage extends StatefulWidget {
  const SongReaderPage({
    super.key,
    required this.input,
    this.ler = lerDoNucleo,
    this.transpor = transposeToKey,
    this.tons = keyChoices,
    this.portaoDePublicacao = conferirPublicacao,
    this.biblioteca,
    this.indiceNaBiblioteca = 0,
    this.diretorioBase,
  });

  /// A cifra em ChordPro, como veio do arquivo ou do exemplo.
  final String input;

  /// De onde vem a estrutura da cifra. Default: núcleo Rust.
  final Future<SongDetail> Function(String input) ler;

  /// Como transpõe. Default: núcleo Rust.
  final Future<SongDetail> Function(String input, String toKey) transpor;

  /// Os tons oferecidos ao músico. Default: núcleo Rust.
  final Future<List<String>> Function() tons;

  /// O portão de publicação do motor. Default: núcleo Rust. É ele que decide
  /// se a letra pode virar link público; a tela não copia a regra.
  final Future<void> Function(String cifra) portaoDePublicacao;

  /// Lista de músicas da biblioteca, para navegação entre cifras.
  final List<SongSummary>? biblioteca;

  /// Índice da música atual na biblioteca.
  final int indiceNaBiblioteca;

  /// Diretório base para carregar cifras da biblioteca.
  final String? diretorioBase;

  @override
  State<SongReaderPage> createState() => _SongReaderPageState();
}

class _SongReaderPageState extends State<SongReaderPage> {
  SongDetail? _cifra;
  List<String> _tons = const <String>[];
  Object? _erro;
  bool _palco = false;
  bool _trocandoTom = false;
  late int _indiceBiblioteca = widget.indiceNaBiblioteca;

  @override
  void initState() {
    super.initState();
    _carregarAtual();
  }

  bool get _temBiblioteca => widget.biblioteca != null && widget.biblioteca!.length > 1;

  bool get _temAnterior => _temBiblioteca && _indiceBiblioteca > 0;

  bool get _temProxima => _temBiblioteca && _indiceBiblioteca < widget.biblioteca!.length - 1;

  Future<void> _carregarAtual() async {
    if (_temBiblioteca) {
      await _carregarDaBiblioteca(_indiceBiblioteca);
    } else {
      await _carregar(widget.input);
    }
  }

  Future<void> _carregarDaBiblioteca(int indice) async {
    final musica = widget.biblioteca![indice];
    final id = musica.title.isNotEmpty
        ? musica.title.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '-')
        : '';
    if (id.isEmpty || widget.diretorioBase == null) {
      setState(() => _erro = 'Música sem identificador');
      return;
    }
    try {
      final chordpro = await _carregarCifra(id);
      await _carregar(chordpro);
    } on Object catch (e) {
      if (!mounted) return;
      setState(() => _erro = e);
    }
  }

  Future<String> _carregarCifra(String id) async {
    // Usa o pacote bridge diretamente
    return carregarDaBiblioteca(baseDir: widget.diretorioBase!, id: id);
  }

  Future<void> _carregar(String chordpro) async {
    try {
      final cifra = await widget.ler(chordpro);
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

  void _anterior() {
    if (!_temAnterior) return;
    setState(() {
      _indiceBiblioteca -= 1;
      _cifra = null;
      _erro = null;
    });
    _carregarDaBiblioteca(_indiceBiblioteca);
  }

  void _proxima() {
    if (!_temProxima) return;
    setState(() {
      _indiceBiblioteca += 1;
      _cifra = null;
      _erro = null;
    });
    _carregarDaBiblioteca(_indiceBiblioteca);
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

  Future<void> _copiarLink() async {
    final l10n = CoreL10n.of(context);
    try {
      // O motor decide se a letra pode circular; a tela só monta o link
      // depois do portão. Escopo privado do grupo não vira link público.
      final link = await linkDeProjecao(
        widget.input,
        portao: widget.portaoDePublicacao,
      );
      await Clipboard.setData(ClipboardData(text: link.toString()));
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.stageProjectionLinkCopied)),
      );
    } on PublicacaoBloqueada catch (bloqueio) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.stageProjectionBlocked(bloqueio.motivo))),
      );
    }
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
        backgroundColor: _palco ? _slateInk : null,
        foregroundColor: _palco ? _paperWhite : null,
        title: Text(titulo),
        leading: _temBiblioteca
            ? Row(
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  IconButton(
                    icon: const Icon(Icons.arrow_back),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                  if (_temAnterior)
                    IconButton(
                      icon: const Icon(Icons.chevron_left),
                      tooltip: 'Anterior',
                      onPressed: _anterior,
                    ),
                  if (_temProxima)
                    IconButton(
                      icon: const Icon(Icons.chevron_right),
                      tooltip: 'Próxima',
                      onPressed: _proxima,
                    ),
                ],
              )
            : null,
        actions: <Widget>[
          if (_temBiblioteca)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Center(
                child: Text(
                  '${_indiceBiblioteca + 1} / ${widget.biblioteca!.length}',
                  style: const TextStyle(fontSize: 14),
                ),
              ),
            ),
          IconButton(
            tooltip: l10n.stageProject,
            onPressed: _cifra == null ? null : _abrirProjecao,
            icon: const Icon(Icons.slideshow),
          ),
          if (kIsWeb)
            IconButton(
              tooltip: l10n.stageCopyProjectionLink,
              onPressed: _cifra == null ? null : _copiarLink,
              icon: const Icon(Icons.link),
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
    final tempo = cifra.time;
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
          if (tempo != null && tempo.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 4, 20, 0),
              child: Row(
                children: <Widget>[
                  Text('${l10n.stageTime}: ', style: _mono.copyWith(color: cor)),
                  Text(tempo, style: _mono.copyWith(color: _vibrantGold)),
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
