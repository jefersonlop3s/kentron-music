import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/erro_do_nucleo.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:material_ui/material_ui.dart';

/// Assinatura de quem grava o evento: ponte no app, dublê no teste.
typedef SalvarEvento =
    Future<EventoAgenda> Function({
      required String baseDir,
      required String id,
      required String nome,
      String? data,
      String? tipo,
      required List<EntradaAgenda> execucoes,
    });

/// Assinatura de quem lista a biblioteca para o seletor de obra.
typedef ListarObras = Future<List<SongSummary>> Function(String baseDir);

/// Grava o evento pelo núcleo. Função de topo porque a tela a recebe por
/// injeção: dublê nos testes herméticos, ponte no app real (mesmo padrão do
/// leitor e da agenda).
Future<EventoAgenda> salvarEventoNoNucleo({
  required String baseDir,
  required String id,
  required String nome,
  String? data,
  String? tipo,
  required List<EntradaAgenda> execucoes,
}) => salvarEventoAgenda(
  baseDir: baseDir,
  id: id,
  nome: nome,
  data: data,
  tipo: tipo,
  execucoes: execucoes,
);

/// Lista as obras da biblioteca para escolher. O erro do núcleo sobe para a
/// tela: biblioteca ilegível **não** vira lista vazia.
Future<List<SongSummary>> listarObrasDaBiblioteca(String baseDir) =>
    listarBiblioteca(baseDir: baseDir);

/// O relógio da tela. Existe como função de topo para o id do evento novo ser
/// reproduzível no teste.
DateTime agora() => DateTime.now();

/// O id do evento novo: o slug do nome mais a data — ou mais o relógio, quando
/// o evento não tem data, para que dois eventos de mesmo nome não se
/// sobrescrevam. O núcleo só aceita um id que sirva de nome de arquivo.
String idDoEvento(String nome, String? data, DateTime agora) {
  final slug = nome
      .toLowerCase()
      .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
      .replaceAll(RegExp(r'^-+|-+$'), '');
  final base = slug.isEmpty ? 'evento' : slug;
  if (data != null && data.isNotEmpty) return '$base-$data';
  String par(int valor) => valor.toString().padLeft(2, '0');
  return '$base-${agora.year}${par(agora.month)}${par(agora.day)}'
      '-${par(agora.hour)}${par(agora.minute)}${par(agora.second)}';
}

/// Monta (ou edita) um evento da agenda: nome, data, tipo e as execuções.
///
/// Grava de verdade pelo núcleo (`salvar_evento_agenda`): o evento só muda no
/// disco quando "Salvar evento" é acionado. Sem [evento] é um evento novo, com
/// id derivado do nome e da data; com [evento], edita o que já existe — mesmo
/// id, mesmo arquivo.
class AgendaEditorPage extends StatefulWidget {
  const AgendaEditorPage({
    super.key,
    this.evento,
    this.diretorioBase,
    this.salvar = salvarEventoNoNucleo,
    this.carregarBiblioteca = listarObrasDaBiblioteca,
    this.tons = keyChoices,
    this.relogio = agora,
  });

  /// O evento a editar. `null` = evento novo.
  final EventoAgenda? evento;

  /// Diretório base da biblioteca e da agenda. Default: o do app.
  final String? diretorioBase;

  /// Como o evento é gravado. Default: núcleo Rust.
  final SalvarEvento salvar;

  /// De onde vem a lista de obras do seletor. Default: núcleo Rust.
  final ListarObras carregarBiblioteca;

  /// Os tons oferecidos em cada execução. Default: núcleo Rust.
  final Future<List<String>> Function() tons;

  /// O relógio do id do evento novo. Injetável para o teste ser determinístico.
  final DateTime Function() relogio;

  @override
  State<AgendaEditorPage> createState() => _AgendaEditorPageState();
}

class _AgendaEditorPageState extends State<AgendaEditorPage> {
  late final TextEditingController _nome;
  late final TextEditingController _data;
  String? _tipo;
  final List<_ExecucaoEditavel> _execucoes = <_ExecucaoEditavel>[];
  List<String> _tons = const <String>[];
  bool _salvando = false;
  String? _erro;

  String get _diretorio => widget.diretorioBase ?? bibliotecaDir();

  bool get _editando => widget.evento != null;

  @override
  void initState() {
    super.initState();
    final EventoAgenda? evento = widget.evento;
    _nome = TextEditingController(text: evento?.nome ?? '');
    _data = TextEditingController(text: evento?.data ?? '');
    _tipo = evento?.tipo;
    for (final ExecucaoAgenda execucao
        in evento?.execucoes ?? const <ExecucaoAgenda>[]) {
      _execucoes.add(
        _ExecucaoEditavel(
          songId: execucao.songId,
          titulo: execucao.titulo,
          versao: execucao.versao,
          tom: execucao.tom,
        ),
      );
    }
    _carregarTons();
  }

  @override
  void dispose() {
    _nome.dispose();
    _data.dispose();
    for (final _ExecucaoEditavel execucao in _execucoes) {
      execucao.versao.dispose();
    }
    super.dispose();
  }

  /// Os tons do seletor vêm do núcleo. Sem eles a tela não inventa uma lista: o
  /// motivo aparece na tela, e o tom continua podendo ficar sem declaração.
  Future<void> _carregarTons() async {
    try {
      final tons = await widget.tons();
      if (!mounted) return;
      setState(() => _tons = tons);
    } on Object catch (erro) {
      if (!mounted) return;
      setState(() => _erro = erro.toString());
    }
  }

  Future<void> _escolherData() async {
    final DateTime hoje = widget.relogio();
    final DateTime atual = DateTime.tryParse(_data.text.trim()) ?? hoje;
    final DateTime? escolhida = await showDatePicker(
      context: context,
      initialDate: atual,
      firstDate: DateTime(hoje.year - 20),
      lastDate: DateTime(hoje.year + 5),
    );
    if (escolhida == null || !mounted) return;
    setState(() => _data.text = _formatarData(escolhida));
  }

  /// Abre o seletor de obra e acrescenta a execução escolhida.
  ///
  /// O tom começa no que a biblioteca declara hoje para a obra: é o tom da obra
  /// agora, e quem monta o evento decide se muda.
  Future<void> _adicionarExecucao() async {
    final SongSummary? escolhida = await showDialog<SongSummary>(
      context: context,
      builder: (BuildContext _) => _SeletorDeObra(
        carregar: widget.carregarBiblioteca,
        diretorio: _diretorio,
        jaNoEvento: _execucoes
            .map((_ExecucaoEditavel execucao) => execucao.songId)
            .toSet(),
      ),
    );
    if (escolhida == null || !mounted) return;
    setState(() {
      _execucoes.add(
        _ExecucaoEditavel(
          songId: escolhida.id,
          titulo: escolhida.title,
          versao: null,
          tom: escolhida.key,
        ),
      );
    });
  }

  void _removerExecucao(int indice) {
    setState(() => _execucoes.removeAt(indice).versao.dispose());
  }

  Future<void> _salvar() async {
    final CoreL10n l10n = CoreL10n.of(context);
    final String nome = _nome.text.trim();
    if (nome.isEmpty) {
      setState(() => _erro = l10n.agendaNameRequired);
      return;
    }
    final String data = _data.text.trim();
    if (data.isNotEmpty && !RegExp(r'^\d{4}-\d{2}-\d{2}$').hasMatch(data)) {
      setState(() => _erro = l10n.agendaDateInvalid);
      return;
    }

    setState(() {
      _salvando = true;
      _erro = null;
    });
    try {
      final EventoAgenda salvo = await widget.salvar(
        baseDir: _diretorio,
        id: _editando
            ? widget.evento!.id
            : idDoEvento(nome, data.isEmpty ? null : data, widget.relogio()),
        nome: nome,
        data: data.isEmpty ? null : data,
        tipo: _tipo,
        execucoes: _execucoes
            .map(
              (_ExecucaoEditavel execucao) => EntradaAgenda(
                songId: execucao.songId,
                versao: _textoOuNulo(execucao.versao.text),
                tom: execucao.tom,
              ),
            )
            .toList(growable: false),
      );
      if (!mounted) return;
      // O evento salvo volta para a agenda: ela recarrega e mostra o que o
      // núcleo gravou, em vez de a tela adivinhar o resultado.
      Navigator.of(context).pop(salvo);
    } on Object catch (erro) {
      if (!mounted) return;
      // Fail-loud: a recusa do núcleo aparece na tela com a mensagem, e o que
      // foi digitado continua ali para ser corrigido.
      setState(() {
        _salvando = false;
        _erro = erro.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final CoreL10n l10n = CoreL10n.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(_editando ? l10n.agendaEditEvent : l10n.agendaNewEvent),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: <Widget>[
          TextField(
            controller: _nome,
            decoration: InputDecoration(
              labelText: l10n.agendaEventName,
              border: const OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 12),
          Row(
            children: <Widget>[
              Expanded(
                child: TextField(
                  controller: _data,
                  decoration: InputDecoration(
                    labelText: l10n.agendaEventDate,
                    hintText: l10n.agendaDateHint,
                    border: const OutlineInputBorder(),
                  ),
                ),
              ),
              IconButton(
                tooltip: l10n.agendaEventDate,
                onPressed: _escolherData,
                icon: const Icon(Icons.calendar_month),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: <Widget>[
              Text(l10n.agendaEventType),
              const SizedBox(width: 12),
              DropdownButton<String>(
                value: _tipo,
                hint: Text(l10n.agendaEventType),
                items: <DropdownMenuItem<String>>[
                  DropdownMenuItem<String>(
                    value: 'culto',
                    child: Text(l10n.agendaTypeService),
                  ),
                  DropdownMenuItem<String>(
                    value: 'ensaio',
                    child: Text(l10n.agendaTypeRehearsal),
                  ),
                  DropdownMenuItem<String>(
                    value: 'evento',
                    child: Text(l10n.agendaTypeEvent),
                  ),
                ],
                onChanged: (String? valor) => setState(() => _tipo = valor),
              ),
            ],
          ),
          const SizedBox(height: 24),
          Text(
            l10n.agendaExecutions,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          if (_execucoes.isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 8),
              child: Text(l10n.agendaNoExecutions),
            ),
          for (int indice = 0; indice < _execucoes.length; indice++)
            _LinhaDaExecucao(
              execucao: _execucoes[indice],
              tons: _tons,
              l10n: l10n,
              onRemover: () => _removerExecucao(indice),
              onTom: (String? tom) =>
                  setState(() => _execucoes[indice].tom = _textoOuNulo(tom)),
            ),
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton.icon(
              onPressed: _adicionarExecucao,
              icon: const Icon(Icons.playlist_add),
              label: Text(l10n.agendaAddExecution),
            ),
          ),
          if (_erro != null)
            AreaDeErro(mensagem: l10n.agendaSaveFailed, erro: _erro!),
          const SizedBox(height: 16),
          FilledButton.icon(
            onPressed: _salvando ? null : _salvar,
            icon: const Icon(Icons.save),
            label: Text(l10n.agendaSaveEvent),
          ),
        ],
      ),
    );
  }
}

/// O seletor de obra: a biblioteca do núcleo, com filtro por título.
///
/// A lista sai da biblioteca **agora** — inclusive o que a obra declara hoje — e
/// o erro do núcleo aparece aqui dentro, com o caminho de recarregar. Obra que já
/// está no evento não é oferecida: o núcleo recusaria a execução repetida.
class _SeletorDeObra extends StatefulWidget {
  const _SeletorDeObra({
    required this.carregar,
    required this.diretorio,
    required this.jaNoEvento,
  });

  final ListarObras carregar;
  final String diretorio;
  final Set<String> jaNoEvento;

  @override
  State<_SeletorDeObra> createState() => _SeletorDeObraState();
}

class _SeletorDeObraState extends State<_SeletorDeObra> {
  List<SongSummary> _obras = const <SongSummary>[];
  final TextEditingController _filtro = TextEditingController();
  bool _carregando = true;
  String? _erro;

  @override
  void initState() {
    super.initState();
    _carregar();
  }

  @override
  void dispose() {
    _filtro.dispose();
    super.dispose();
  }

  Future<void> _carregar() async {
    setState(() {
      _carregando = true;
      _erro = null;
    });
    try {
      final obras = await widget.carregar(widget.diretorio);
      if (!mounted) return;
      setState(() {
        _obras = obras;
        _carregando = false;
      });
    } on Object catch (erro) {
      if (!mounted) return;
      setState(() {
        _obras = const <SongSummary>[];
        _erro = erro.toString();
        _carregando = false;
      });
    }
  }

  List<SongSummary> get _visiveis {
    final String filtro = _filtro.text.trim().toLowerCase();
    return _obras
        .where(
          (SongSummary obra) => !widget.jaNoEvento.contains(obra.id),
        )
        .where(
          (SongSummary obra) =>
              filtro.isEmpty || obra.title.toLowerCase().contains(filtro),
        )
        .toList(growable: false);
  }

  @override
  Widget build(BuildContext context) {
    final CoreL10n l10n = CoreL10n.of(context);
    return AlertDialog(
      title: Text(l10n.agendaPickSong),
      content: SizedBox(width: double.maxFinite, height: 360, child: _corpo(l10n)),
      actions: <Widget>[
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.agendaCancel),
        ),
      ],
    );
  }

  Widget _corpo(CoreL10n l10n) {
    if (_carregando) return const Center(child: CircularProgressIndicator());
    if (_erro != null) {
      return SingleChildScrollView(
        child: AreaDeErro(
          mensagem: l10n.agendaPickerFailed,
          erro: _erro!,
          onTentarNovamente: _carregar,
        ),
      );
    }

    final List<SongSummary> visiveis = _visiveis;
    return Column(
      children: <Widget>[
        TextField(
          controller: _filtro,
          onChanged: (String _) => setState(() {}),
          decoration: InputDecoration(
            labelText: l10n.agendaFilterSongs,
            border: const OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 8),
        if (_obras.isEmpty)
          Expanded(child: Center(child: Text(l10n.agendaPickerEmpty)))
        else if (visiveis.isEmpty)
          // Biblioteca cheia e filtro sem resultado não é "biblioteca vazia".
          Expanded(child: Center(child: Text(l10n.agendaPickerNoMatch)))
        else
          Expanded(
            child: ListView.builder(
              itemCount: visiveis.length,
              itemBuilder: (BuildContext context, int indice) {
                final SongSummary obra = visiveis[indice];
                return ListTile(
                  leading: const Icon(Icons.music_note),
                  title: Text(obra.title.isEmpty ? obra.id : obra.title),
                  // Sem prosa: o id identifica a obra e o tom vem do bundle.
                  subtitle: Text(
                    obra.key == null ? obra.id : '${obra.id} · ${obra.key}',
                  ),
                  onTap: () => Navigator.of(context).pop(obra),
                );
              },
            ),
          ),
      ],
    );
  }
}

String? _textoOuNulo(String? texto) {
  final String limpo = (texto ?? '').trim();
  return limpo.isEmpty ? null : limpo;
}

/// Uma execução em edição: a obra, o campo de versão e o tom escolhido.
class _ExecucaoEditavel {
  _ExecucaoEditavel({
    required this.songId,
    required this.titulo,
    String? versao,
    this.tom,
  }) : versao = TextEditingController(text: versao ?? '');

  /// O id da obra na biblioteca — é ele que o núcleo grava na execução.
  final String songId;

  /// Título da obra, só para a tela: o núcleo lê o título do bundle.
  final String titulo;

  /// O campo de versão. Vazio é "não declarada" — o núcleo registra a ausência.
  final TextEditingController versao;

  /// Tom declarado para o evento, ou `null` para "não declarado".
  String? tom;
}

/// Uma execução na lista: obra, versão e o tom daquele evento.
class _LinhaDaExecucao extends StatelessWidget {
  const _LinhaDaExecucao({
    required this.execucao,
    required this.tons,
    required this.l10n,
    required this.onRemover,
    required this.onTom,
  });

  final _ExecucaoEditavel execucao;
  final List<String> tons;
  final CoreL10n l10n;
  final VoidCallback onRemover;
  final ValueChanged<String?> onTom;

  @override
  Widget build(BuildContext context) {
    // O valor corrente **tem** de estar entre os itens: um evento antigo pode
    // declarar um tom que a lista de hoje não traz (`C#`), e o seletor não pode
    // estourar por isso.
    final String tom = execucao.tom ?? '';
    final List<String> valores = <String>[
      '',
      if (!tons.contains(tom) && tom.isNotEmpty) tom,
      ...tons,
    ];
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
                    execucao.titulo.isEmpty ? execucao.songId : execucao.titulo,
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                ),
                IconButton(
                  tooltip: l10n.agendaExecutionRemove,
                  onPressed: onRemover,
                  icon: const Icon(Icons.delete_outline),
                ),
              ],
            ),
            TextField(
              controller: execucao.versao,
              decoration: InputDecoration(
                labelText: l10n.agendaExecutionVersion,
                border: const OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: <Widget>[
                Text(l10n.agendaExecutionKey),
                const SizedBox(width: 12),
                DropdownButton<String>(
                  value: tom,
                  items: <DropdownMenuItem<String>>[
                    for (final String valor in valores)
                      DropdownMenuItem<String>(
                        value: valor,
                        // Vazio é "não declarado": um traço, sem inventar tom.
                        child: Text(valor.isEmpty ? '—' : valor),
                      ),
                  ],
                  onChanged: onTom,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

String _formatarData(DateTime data) =>
    '${data.year.toString().padLeft(4, '0')}-'
    '${data.month.toString().padLeft(2, '0')}-'
    '${data.day.toString().padLeft(2, '0')}';
