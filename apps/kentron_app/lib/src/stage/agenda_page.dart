import 'package:core_l10n/core_l10n.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/agenda_editor_page.dart';
import 'package:kentron_app/src/stage/erro_do_nucleo.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:kentron_app/src/stage/uso_page.dart';
import 'package:material_ui/material_ui.dart';

/// Lê a agenda pelo núcleo. Função de topo porque a tela a recebe por injeção:
/// dublê nos testes herméticos, ponte no app real (mesmo padrão do leitor).
Future<List<EventoAgenda>> lerAgendaDoNucleo(String baseDir) =>
    listarAgenda(baseDir: baseDir);

/// Repete a agenda anterior pelo núcleo (duplica o último evento).
Future<EventoAgenda> repetirAgendaNoNucleo({
  required String baseDir,
  required String novoId,
  required String nome,
  String? data,
}) => repetirUltimoEvento(
  baseDir: baseDir,
  novoId: novoId,
  nome: nome,
  data: data,
);

/// Assinatura de quem repete o evento anterior: ponte no app, dublê no teste.
typedef RepetirEvento =
    Future<EventoAgenda> Function({
      required String baseDir,
      required String novoId,
      required String nome,
      String? data,
    });

/// Abre um setlist no Modo Palco: o leitor carrega cada obra **pelo id** e o
/// diretório base diz de onde; a ordem é a da agenda.
Widget abrirSetlistNoPalco(List<String> ids, String diretorio) =>
    SongReaderPage(
      input: '',
      idsBiblioteca: ids,
      diretorioBase: diretorio,
    );

/// Abre o editor de evento de verdade (o que chama a ponte na montagem para os
/// tons). Injetável para o teste de widget da agenda não depender de FFI.
Widget abrirEditorDoNucleo(EventoAgenda? evento, String diretorio) =>
    AgendaEditorPage(evento: evento, diretorioBase: diretorio);

/// A agenda local: um evento por linha, com data, tipo e o setlist dele.
///
/// A ordem é a do núcleo (nome do arquivo da agenda), não uma ordenação
/// inventada na tela. Toque no evento: o setlist abre no Modo Palco, música por
/// música, na ordem da agenda.
///
/// Estados: carregando, erro do núcleo visível (fail-loud — nunca vira lista
/// vazia) e vazio, que significa, e só significa, biblioteca sem agenda.
class AgendaPage extends StatefulWidget {
  const AgendaPage({
    super.key,
    this.carregarAgenda = lerAgendaDoNucleo,
    this.diretorioBase,
    this.abrirSetlist = abrirSetlistNoPalco,
    this.abrirEditor = abrirEditorDoNucleo,
    this.repetir = repetirAgendaNoNucleo,
    this.relogio = agora,
  });

  /// De onde vem a agenda. Default: núcleo Rust.
  final Future<List<EventoAgenda>> Function(String baseDir) carregarAgenda;

  /// Diretório base da biblioteca e da agenda. Default: o do app.
  final String? diretorioBase;

  /// Como o setlist do evento é aberto. Default: o Modo Palco de verdade, com o
  /// núcleo respondendo. Injetável porque o leitor chama a ponte já na montagem
  /// (tom, tons do seletor) — o teste de widget da agenda não deve depender de FFI.
  final Widget Function(List<String> ids, String diretorio) abrirSetlist;

  /// Como o editor do evento é aberto. Default: a tela de edição de verdade.
  /// Injetável pelo mesmo motivo do leitor (o editor pede os tons ao núcleo).
  final Widget Function(EventoAgenda? evento, String diretorio) abrirEditor;

  /// Como o evento anterior é repetido. Default: núcleo Rust.
  final RepetirEvento repetir;

  /// O relógio do id do evento repetido. Injetável para o teste ser determinístico.
  final DateTime Function() relogio;

  @override
  State<AgendaPage> createState() => _AgendaPageState();
}

class _AgendaPageState extends State<AgendaPage> {
  List<EventoAgenda> _eventos = const <EventoAgenda>[];
  bool _carregando = true;
  String? _erro;

  /// Diretório base da agenda e da biblioteca (mesmo caminho do app).
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
      final eventos = await widget.carregarAgenda(_diretorio);
      if (!mounted) return;
      setState(() {
        _eventos = eventos;
        _carregando = false;
      });
    } on Object catch (erro) {
      if (!mounted) return;
      // Fail-loud: a falha do núcleo aparece na tela com a mensagem — nunca vira
      // a lista vazia, que significa "biblioteca sem agenda".
      setState(() {
        _eventos = const <EventoAgenda>[];
        _erro = erro.toString();
        _carregando = false;
      });
    }
  }

  /// Abre o setlist do evento no Modo Palco, na ordem da agenda.
  ///
  /// Os ids vêm do núcleo e são repassados como estão: o leitor não adivinha
  /// identificador por título. Evento sem música não abre nada.
  void _abrirNoPalco(EventoAgenda evento) {
    final ids = evento.execucoes
        .map((ExecucaoAgenda execucao) => execucao.songId)
        .toList(growable: false);
    if (ids.isEmpty) return;
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => widget.abrirSetlist(ids, _diretorio),
      ),
    );
  }

  /// Abre o editor: sem evento, cria um; com evento, edita o que já existe.
  ///
  /// O evento salvo volta do editor e a agenda recarrega: a lista passa a mostrar
  /// o que o núcleo gravou, não o que a tela supõe.
  Future<void> _abrirEditor(EventoAgenda? evento) async {
    final CoreL10n l10n = CoreL10n.of(context);
    final EventoAgenda? salvo = await Navigator.of(context).push<EventoAgenda>(
      MaterialPageRoute<EventoAgenda>(
        builder: (_) => widget.abrirEditor(evento, _diretorio),
      ),
    );
    if (salvo == null || !mounted) return;
    await _recarregar();
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(l10n.agendaEventSaved)));
  }

  /// Repete a agenda anterior: duplica o **último evento da lista** com as
  /// versões e tons de agora da biblioteca.
  ///
  /// Sem evento na lista não há anterior: a tela diz isso em vez de criar um
  /// evento vazio. E a recusa do núcleo (id tomado, agenda ilegível) aparece na
  /// tela com a mensagem dele — repetir nunca falha em silêncio.
  Future<void> _repetirAnterior() async {
    final CoreL10n l10n = CoreL10n.of(context);
    if (_eventos.isEmpty) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(l10n.agendaNoPreviousEvent)));
      return;
    }
    final EventoAgenda anterior = _eventos.last;
    try {
      await widget.repetir(
        baseDir: _diretorio,
        novoId: idDoEvento(anterior.nome, null, widget.relogio()),
        nome: anterior.nome,
      );
      if (!mounted) return;
      await _recarregar();
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(l10n.agendaRepeatDone)));
    } on Object catch (erro) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('${l10n.agendaRepeatFailed}: $erro')),
      );
    }
  }

  void _abrirRelatorio() {
    Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => UsoPage(diretorioBase: _diretorio)));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = CoreL10n.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.agendaTitle),
        actions: <Widget>[
          IconButton(
            tooltip: l10n.agendaRepeatPrevious,
            onPressed: _repetirAnterior,
            icon: const Icon(Icons.copy_all),
          ),
          IconButton(
            tooltip: l10n.usoTitle,
            onPressed: _abrirRelatorio,
            icon: const Icon(Icons.insights),
          ),
        ],
      ),
      body: _corpo(l10n),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _abrirEditor(null),
        icon: const Icon(Icons.add),
        label: Text(l10n.agendaNewEvent),
      ),
    );
  }

  Widget _corpo(CoreL10n l10n) {
    if (_carregando) return const Center(child: CircularProgressIndicator());

    if (_erro != null) {
      return Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: AreaDeErro(
            mensagem: l10n.agendaLoadFailed,
            erro: _erro!,
            onTentarNovamente: _recarregar,
          ),
        ),
      );
    }

    if (_eventos.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const Icon(Icons.event_note, size: 64),
              const SizedBox(height: 16),
              Text(l10n.agendaEmpty, textAlign: TextAlign.center),
            ],
          ),
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _recarregar,
      child: ListView.builder(
        itemCount: _eventos.length,
        itemBuilder: (context, indice) => _EventoDaAgenda(
          evento: _eventos[indice],
          l10n: l10n,
          onAbrir: _abrirNoPalco,
          onEditar: () => _abrirEditor(_eventos[indice]),
        ),
      ),
    );
  }
}

/// Uma linha da agenda: nome do evento e, abaixo, data, tipo e nº de músicas.
class _EventoDaAgenda extends StatelessWidget {
  const _EventoDaAgenda({
    required this.evento,
    required this.l10n,
    required this.onAbrir,
    required this.onEditar,
  });

  final EventoAgenda evento;
  final CoreL10n l10n;
  final void Function(EventoAgenda) onAbrir;
  final VoidCallback onEditar;

  @override
  Widget build(BuildContext context) {
    final total = evento.execucoes.length;
    final detalhe = <String>[
      evento.data ?? l10n.agendaNoDate,
      evento.tipo ?? l10n.agendaNoType,
      l10n.setlistSongCount(total),
    ].join(' · ');
    return ListTile(
      leading: Icon(_iconeDoTipo(evento.tipo)),
      title: Text(evento.nome),
      subtitle: Text(detalhe),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          IconButton(
            tooltip: l10n.agendaEditEvent,
            onPressed: onEditar,
            icon: const Icon(Icons.edit_outlined),
          ),
          const Icon(Icons.chevron_right),
        ],
      ),
      // Sem música não há setlist a abrir: o toque fica inerte em vez de abrir
      // um leitor sem cifra. Editar continua disponível — é como se acrescenta
      // música a um evento que nasceu vazio.
      onTap: total == 0 ? null : () => onAbrir(evento),
    );
  }
}

IconData _iconeDoTipo(String? tipo) {
  switch (tipo) {
    case 'culto':
      return Icons.church;
    case 'ensaio':
      return Icons.graphic_eq;
    default:
      return Icons.event;
  }
}
