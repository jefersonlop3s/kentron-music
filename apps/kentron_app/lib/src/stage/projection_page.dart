import 'package:core_l10n/core_l10n.dart';
import 'package:flutter/services.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:material_ui/material_ui.dart';

/// Tokens do DESIGN.md usados na projeção.
const _slateInk = Color(0xFF0B0F14);
const _paperWhite = Color(0xFFF2F5F7);
const _vibrantGold = Color(0xFFE0B341);

/// Número máximo de linhas de letra visíveis por página de projeção.
/// Acima disso, a seção é dividida em sub-páginas.
const _linhasPorPagina = 6;

/// Projeção manual: uma seção por vez, letra grande, sem acorde.
///
/// "Manual" é literal: o operador avança e volta. Não há rolagem automática
/// nem cronômetro — quem projeta decide o ritmo. A letra é o que a congregação
/// lê; o acorde fica no Modo Palco, com a banda.
///
/// Seções com mais de [_linhasPorPagina] linhas são automaticamente divididas
/// em sub-páginas para caber na tela sem scroll.
///
/// O avanço tem três caminhos: toque na metade direita/esquerda da tela,
/// setas do teclado e os botões do rodapé. Todos chamam o mesmo par de métodos,
/// então não há caminho que pule estado.
class ProjectionPage extends StatefulWidget {
  const ProjectionPage({
    super.key,
    required this.song,
    this.initialSection = 0,
    this.initialSubPage = 0,
  });

  /// A cifra já lida pelo núcleo — a projeção não relê o texto.
  final SongDetail song;

  /// Seção inicial; é limitada ao intervalo válido.
  final int initialSection;

  /// Sub-página inicial dentro da seção (para seções longas).
  final int initialSubPage;

  @override
  State<ProjectionPage> createState() => _ProjectionPageState();
}

class _ProjectionPageState extends State<ProjectionPage> {
  late int _indice = _limitarSecao(widget.initialSection);
  late int _subPagina = widget.initialSubPage;

  int _limitarSecao(int indice) {
    final total = widget.song.sections.length;
    if (total == 0) return 0;
    return indice.clamp(0, total - 1);
  }

  /// Número de sub-páginas na seção atual.
  int get _totalSubPaginas {
    final linhas = _linhasDaSecaoAtual;
    if (linhas <= _linhasPorPagina) return 1;
    return (linhas + _linhasPorPagina - 1) ~/ _linhasPorPagina;
  }

  /// Linhas de letra (não vazias) da seção atual.
  int get _linhasDaSecaoAtual {
    final secao = widget.song.sections[_indice];
    return secao.lines.where((l) => l.text.isNotEmpty).length;
  }

  bool get _temAnterior =>
      _indice > 0 || (_subPagina > 0 && _totalSubPaginas > 1);

  bool get _temProxima =>
      _indice < widget.song.sections.length - 1 ||
      (_subPagina < _totalSubPaginas - 1 && _totalSubPaginas > 1);

  void _anterior() {
    if (_subPagina > 0 && _totalSubPaginas > 1) {
      setState(() => _subPagina -= 1);
    } else if (_indice > 0) {
      setState(() {
        _indice -= 1;
        _subPagina = _totalSubPaginas - 1;
      });
    }
  }

  void _proxima() {
    if (_subPagina < _totalSubPaginas - 1 && _totalSubPaginas > 1) {
      setState(() => _subPagina += 1);
    } else if (_indice < widget.song.sections.length - 1) {
      setState(() {
        _indice += 1;
        _subPagina = 0;
      });
    }
  }

  void _tocar(Offset posicao, double largura) {
    if (posicao.dx < largura / 2) {
      _anterior();
    } else {
      _proxima();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = CoreL10n.of(context);
    final secoes = widget.song.sections;
    if (secoes.isEmpty) {
      return const Scaffold(
        backgroundColor: _slateInk,
        body: Center(
          child: Text('—', style: TextStyle(color: _paperWhite, fontSize: 64)),
        ),
      );
    }
    final secao = secoes[_indice];
    return Scaffold(
      backgroundColor: _slateInk,
      body: CallbackShortcuts(
        bindings: <ShortcutActivator, VoidCallback>{
          const SingleActivator(LogicalKeyboardKey.arrowRight): _proxima,
          const SingleActivator(LogicalKeyboardKey.arrowLeft): _anterior,
        },
        child: Focus(
          autofocus: true,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapUp: (detalhe) => _tocar(
              detalhe.localPosition,
              MediaQuery.sizeOf(context).width,
            ),
            child: SafeArea(
              child: Column(
                children: <Widget>[
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 48,
                        vertical: 32,
                      ),
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: _ConteudoDaSecao(
                          secao: secao,
                          subPagina: _subPagina,
                          linhasPorPagina: _linhasPorPagina,
                        ),
                      ),
                    ),
                  ),
                  _Rodape(
                    l10n: l10n,
                    indice: _indice,
                    total: secoes.length,
                    subPagina: _totalSubPaginas > 1 ? _subPagina : null,
                    totalSubPaginas: _totalSubPaginas > 1 ? _totalSubPaginas : null,
                    temAnterior: _temAnterior,
                    temProxima: _temProxima,
                    onAnterior: _anterior,
                    onProxima: _proxima,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// O que a congregação lê: nome da seção e letra, sem acorde.
///
/// Quando a seção tem mais de [linhasPorPagina] linhas, só as linhas da
/// [subPagina] atual são exibidas. A paginação é automática e transparente
/// para o operador — ele só vê "avançar" e "voltar".
class _ConteudoDaSecao extends StatelessWidget {
  const _ConteudoDaSecao({
    required this.secao,
    required this.subPagina,
    required this.linhasPorPagina,
  });

  final SongSection secao;
  final int subPagina;
  final int linhasPorPagina;

  @override
  Widget build(BuildContext context) {
    final nome = secao.name;
    final linhas = secao.lines
        .where((l) => l.text.isNotEmpty)
        .toList(growable: false);
    final inicio = subPagina * linhasPorPagina;
    final linhasDaPagina = linhas.skip(inicio).take(linhasPorPagina);

    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.center,
        children: <Widget>[
          if (nome != null && nome.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 24),
              child: Text(
                nome,
                style: const TextStyle(
                  color: _vibrantGold,
                  fontSize: 28,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 1.2,
                ),
              ),
            ),
          for (final linha in linhasDaPagina)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Text(
                linha.text,
                style: const TextStyle(
                  color: _paperWhite,
                  fontSize: 44,
                  height: 1.25,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// Rodapé discreto: posição, sub-página (quando aplicável) e avanço.
/// Alvos de 48 px, como manda o DESIGN.md.
class _Rodape extends StatelessWidget {
  const _Rodape({
    required this.l10n,
    required this.indice,
    required this.total,
    this.subPagina,
    this.totalSubPaginas,
    required this.temAnterior,
    required this.temProxima,
    required this.onAnterior,
    required this.onProxima,
  });

  final CoreL10n l10n;
  final int indice;
  final int total;
  final int? subPagina;
  final int? totalSubPaginas;
  final bool temAnterior;
  final bool temProxima;
  final VoidCallback onAnterior;
  final VoidCallback onProxima;

  @override
  Widget build(BuildContext context) {
    final posicao = subPagina != null
        ? '${indice + 1} / $total · ${subPagina! + 1} / $totalSubPaginas'
        : '${indice + 1} / $total';

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: <Widget>[
          IconButton(
            tooltip: l10n.stageProjectPrevious,
            onPressed: temAnterior ? onAnterior : null,
            icon: const Icon(Icons.chevron_left),
            color: _paperWhite,
            iconSize: 32,
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              posicao,
              style: const TextStyle(color: _paperWhite, fontSize: 16),
            ),
          ),
          IconButton(
            tooltip: l10n.stageProjectNext,
            onPressed: temProxima ? onProxima : null,
            icon: const Icon(Icons.chevron_right),
            color: _paperWhite,
            iconSize: 32,
          ),
        ],
      ),
    );
  }
}
