import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/erro_do_nucleo.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:material_ui/material_ui.dart';

/// Carrega o relatório de uso pelo núcleo (`mir_engine::uso`). Função de topo
/// porque a tela a recebe por injeção: dublê nos testes herméticos, ponte no
/// app real.
Future<RelatorioDeUso> carregarRelatorioDeUso(String baseDir, int anoCorrente) =>
    relatorioDeUso(baseDir: baseDir, anoCorrente: anoCorrente);

/// O ano corrente do relógio do aparelho.
///
/// Vem de fora do núcleo de propósito: o prazo de domínio público (vida + 70) é
/// contado a partir de um ano, e o núcleo não tem relógio. Aqui ele é injetável,
/// então o teste não depende da data em que roda.
int anoCorrenteDeHoje() => DateTime.now().year;

/// Assinatura de quem monta o relatório: ponte no app, dublê no teste.
typedef CarregarRelatorio =
    Future<RelatorioDeUso> Function(String baseDir, int anoCorrente);

/// O relatório de uso: o que a equipe cantou e o que ainda está **sem ficha**.
///
/// É o `--relatorio-uso` do motor na tela: a classificação por ficha (ADR-0012)
/// continua sendo do motor — a tela não decide licença, mostra o veredito.
///
/// Estados: carregando, erro do núcleo visível (fail-loud — nunca vira relatório
/// vazio) e vazio, que significa, e só significa, agenda sem execução.
class UsoPage extends StatefulWidget {
  const UsoPage({
    super.key,
    this.diretorioBase,
    this.carregar = carregarRelatorioDeUso,
    this.anoCorrente = anoCorrenteDeHoje,
  });

  /// Diretório base da biblioteca e da agenda. Default: o do app.
  final String? diretorioBase;

  /// De onde vem o relatório. Default: núcleo Rust.
  final CarregarRelatorio carregar;

  /// O ano que o núcleo usa para o prazo de domínio público. Default: o de hoje.
  final int Function() anoCorrente;

  @override
  State<UsoPage> createState() => _UsoPageState();
}

class _UsoPageState extends State<UsoPage> {
  RelatorioDeUso? _relatorio;
  bool _carregando = true;
  String? _erro;

  String get _diretorio => widget.diretorioBase ?? bibliotecaDir();

  @override
  void initState() {
    super.initState();
    _recarregar();
  }

  Future<void> _recarregar() async {
    setState(() {
      _carregando = true;
      _erro = null;
    });
    try {
      final relatorio = await widget.carregar(_diretorio, widget.anoCorrente());
      if (!mounted) return;
      setState(() {
        _relatorio = relatorio;
        _carregando = false;
      });
    } on Object catch (erro) {
      if (!mounted) return;
      // Fail-loud: a falha do núcleo aparece com a mensagem dele. Relatório sem
      // obra é agenda sem execução — não pode ser o disfarce de um erro.
      setState(() {
        _relatorio = null;
        _erro = erro.toString();
        _carregando = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final CoreL10n l10n = CoreL10n.of(context);
    return Scaffold(
      appBar: AppBar(title: Text(l10n.usoTitle)),
      body: _corpo(l10n),
    );
  }

  Widget _corpo(CoreL10n l10n) {
    if (_carregando) return const Center(child: CircularProgressIndicator());

    if (_erro != null) {
      return Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: AreaDeErro(
            mensagem: l10n.usoLoadFailed,
            erro: _erro!,
            onTentarNovamente: _recarregar,
          ),
        ),
      );
    }

    final RelatorioDeUso? relatorio = _relatorio;
    if (relatorio == null || relatorio.obras.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const Icon(Icons.insights, size: 64),
              const SizedBox(height: 16),
              Text(l10n.usoEmpty, textAlign: TextAlign.center),
            ],
          ),
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _recarregar,
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: <Widget>[
          Text(l10n.usoEvents(relatorio.eventos)),
          Text(l10n.usoExecutions(relatorio.execucoes)),
          Text(l10n.usoWorks(relatorio.obrasDistintas)),
          Text(l10n.usoPublicWorks(relatorio.publicas)),
          Text(l10n.usoWorksWithoutRecord(relatorio.semFicha)),
          const Divider(height: 32),
          for (final ObraDoRelatorio obra in relatorio.obras)
            _LinhaDaObra(obra: obra, l10n: l10n),
          const SizedBox(height: 16),
          Text(
            'núcleo: ${relatorio.core}',
            style: Theme.of(context).textTheme.labelSmall,
          ),
        ],
      ),
    );
  }
}

/// Uma obra do relatório: quantas vezes foi cantada, em que versões e o veredito.
class _LinhaDaObra extends StatelessWidget {
  const _LinhaDaObra({required this.obra, required this.l10n});

  final ObraDoRelatorio obra;
  final CoreL10n l10n;

  @override
  Widget build(BuildContext context) {
    final String? veredito = obra.veredito;
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 6),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Row(
              children: <Widget>[
                Expanded(
                  child: Text(
                    obra.titulo,
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                ),
                Text('${obra.execucoes}×'),
              ],
            ),
            const SizedBox(height: 4),
            for (final VersaoDaObra versao in obra.versoes)
              Text('${versao.vezes}× ${versao.versao}'),
            const SizedBox(height: 8),
            if (veredito == null)
              Chip(
                avatar: const Icon(Icons.help_outline, size: 18),
                label: Text(l10n.usoSemFicha),
              )
            else
              Row(
                children: <Widget>[
                  Icon(
                    obra.publica ? Icons.lock_open : Icons.lock_outline,
                    size: 18,
                  ),
                  const SizedBox(width: 8),
                  Expanded(child: Text(veredito)),
                ],
              ),
          ],
        ),
      ),
    );
  }
}
