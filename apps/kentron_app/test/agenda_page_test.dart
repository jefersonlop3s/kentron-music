import 'dart:async';

import 'package:core_l10n/core_l10n.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/agenda_page.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:kentron_app/src/stage/uso_page.dart';
import 'package:material_ui/material_ui.dart';

/// Eventos de mentira: o teste não toca em FFI, só na lógica de tela. A prova
/// com o núcleo de verdade vive no teste Rust da ponte e em `bridge_test.dart`.
const _culto = EventoAgenda(
  id: 'culto-1',
  nome: 'Culto da manhã',
  data: '2026-11-22',
  tipo: 'culto',
  execucoes: <ExecucaoAgenda>[
    ExecucaoAgenda(songId: 'santo', titulo: 'Santo'),
    ExecucaoAgenda(
      songId: 'arranjo-coral',
      titulo: 'Santo (arranjo do coral)',
      versao: 'versão do coral',
      tom: 'D',
    ),
  ],
  core: 'falso 0.0.0',
);

/// Evento sem data e sem tipo declarados: a tela diz o que falta.
const _ensaio = EventoAgenda(
  id: 'ensaio-0',
  nome: 'Ensaio de quinta',
  execucoes: <ExecucaoAgenda>[
    ExecucaoAgenda(songId: 'hino-3', titulo: 'Hino 3'),
  ],
  core: 'falso 0.0.0',
);

/// Evento sem música: não há setlist a abrir no Modo Palco.
const _semMusica = EventoAgenda(
  id: 'vazio',
  nome: 'Evento sem música',
  data: '2026-12-01',
  tipo: 'evento',
  execucoes: <ExecucaoAgenda>[],
  core: 'falso 0.0.0',
);

Future<List<EventoAgenda>> _agenda(List<EventoAgenda> eventos) async => eventos;

Widget _montar(Widget filho) => MaterialApp(
  locale: const Locale('en'),
  localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
    CoreL10n.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: CoreL10n.supportedLocales,
  home: filho,
);

void main() {
  testWidgets('mostra o indicador enquanto o núcleo não responde', (tester) async {
    final resposta = Completer<List<EventoAgenda>>();
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => resposta.future,
        ),
      ),
    );
    await tester.pump();

    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    resposta.complete(const <EventoAgenda>[]);
    await tester.pumpAndSettle();
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('vazio: biblioteca sem agenda é o estado sem eventos', (tester) async {
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('No events in the agenda yet.'), findsOneWidget);
  });

  testWidgets('o erro do núcleo aparece na tela em vez de virar lista vazia', (
    tester,
  ) async {
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) async =>
              throw StateError('agenda corrompida em culto-1.json'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Could not load the agenda'), findsOneWidget);
    expect(
      find.textContaining('agenda corrompida em culto-1.json'),
      findsOneWidget,
    );
    // O estado vazio não pode aparecer no lugar do erro.
    expect(find.text('No events in the agenda yet.'), findsNothing);
  });

  testWidgets('lista os eventos com data, tipo e número de músicas', (tester) async {
    String? pedido;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String baseDir) async {
            pedido = baseDir;
            return _agenda(const <EventoAgenda>[_culto, _ensaio]);
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(pedido, '/tmp/biblioteca');
    expect(find.text('Culto da manhã'), findsOneWidget);
    expect(find.text('2026-11-22 · culto · 2 songs'), findsOneWidget);
    expect(find.text('Ensaio de quinta'), findsOneWidget);
    expect(find.text('no date · no type · 1 song'), findsOneWidget);
  });

  testWidgets('o toque abre o setlist no Modo Palco, pelo id da agenda', (
    tester,
  ) async {
    String? idsPedidos;
    String? diretorioPedido;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_culto]),
          abrirSetlist: (List<String> ids, String diretorio) {
            idsPedidos = ids.join(',');
            diretorioPedido = diretorio;
            return const Scaffold(body: Text('Modo Palco'));
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Culto da manhã'));
    await tester.pumpAndSettle();

    // O setlist vai na ordem da agenda e pelos ids do núcleo (`santo`,
    // `arranjo-coral`), não pelo slug do título das obras.
    expect(idsPedidos, 'santo,arranjo-coral');
    expect(diretorioPedido, '/tmp/biblioteca');
    expect(find.text('Modo Palco'), findsOneWidget);
  });

  testWidgets('abrir o setlist usa o diretório do app quando não é informado', (
    tester,
  ) async {
    String? diretorioPedido;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_culto]),
          abrirSetlist: (List<String> _, String diretorio) {
            diretorioPedido = diretorio;
            return const Scaffold(body: Text('Modo Palco'));
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Culto da manhã'));
    await tester.pumpAndSettle();

    // O mesmo diretório que a LibraryPage usa (`bibliotecaDir()`).
    expect(diretorioPedido, bibliotecaDir());
    expect(diretorioPedido, isNot(''));
  });

  testWidgets('evento sem música não abre o leitor', (tester) async {
    var abriu = false;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_semMusica]),
          abrirSetlist: (List<String> _, String __) {
            abriu = true;
            return const Scaffold(body: Text('Modo Palco'));
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Evento sem música'));
    await tester.pumpAndSettle();

    expect(abriu, isFalse);
    expect(find.text('Modo Palco'), findsNothing);
  });

  testWidgets('a falha da agenda em português também aparece na tela', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        locale: const Locale('pt'),
        localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
          CoreL10n.delegate,
          ...GlobalMaterialLocalizations.delegates,
        ],
        supportedLocales: CoreL10n.supportedLocales,
        home: AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) async => throw StateError('sem library.json'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Não foi possível carregar a agenda'), findsOneWidget);
    expect(find.textContaining('sem library.json'), findsOneWidget);
  });

  // ── Montar a agenda: o editor e o atalho de repetir ─────────────────────

  /// O evento que o atalho de repetir cria, como o núcleo o devolveria.
  const repetido = EventoAgenda(
    id: 'ensaio-de-quinta-20261129-200000',
    nome: 'Ensaio de quinta',
    data: '2026-11-29',
    tipo: 'ensaio',
    execucoes: <ExecucaoAgenda>[
      ExecucaoAgenda(songId: 'hino-3', titulo: 'Hino 3'),
    ],
    core: 'falso 0.0.0',
  );

  testWidgets('o botão de novo evento abre o editor e a lista recarrega', (
    tester,
  ) async {
    var carregamentos = 0;
    EventoAgenda? pedidoAoEditor;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) async {
            carregamentos++;
            return carregamentos == 1
                ? const <EventoAgenda>[]
                : const <EventoAgenda>[_culto];
          },
          abrirEditor: (EventoAgenda? evento, String _) {
            pedidoAoEditor = evento;
            return Scaffold(
              body: Builder(
                builder: (BuildContext contexto) => Center(
                  child: TextButton(
                    onPressed: () => Navigator.of(contexto).pop(_culto),
                    child: const Text('gravar o evento'),
                  ),
                ),
              ),
            );
          },
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('No events in the agenda yet.'), findsOneWidget);

    await tester.tap(find.text('New event'));
    await tester.pumpAndSettle();
    expect(pedidoAoEditor, isNull, reason: 'evento novo: nada a pré-preencher');

    await tester.tap(find.text('gravar o evento'));
    await tester.pumpAndSettle();

    // A agenda recarrega do núcleo e mostra o que foi salvo.
    expect(carregamentos, 2);
    expect(find.text('Event saved'), findsOneWidget);
    expect(find.text('Culto da manhã'), findsOneWidget);
  });

  testWidgets('a lápide do evento abre o editor com ele preenchido', (
    tester,
  ) async {
    EventoAgenda? pedidoAoEditor;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_culto]),
          abrirEditor: (EventoAgenda? evento, String _) {
            pedidoAoEditor = evento;
            return const Scaffold(body: Text('editor do evento'));
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Edit event'));
    await tester.pumpAndSettle();

    expect(find.text('editor do evento'), findsOneWidget);
    expect(pedidoAoEditor?.id, 'culto-1');
    expect(
      pedidoAoEditor?.execucoes.map((ExecucaoAgenda e) => e.songId),
      <String>['santo', 'arranjo-coral'],
      reason: 'o editor recebe as execuções na ordem da agenda',
    );
  });

  testWidgets('o relatório de uso abre pela agenda', (tester) async {
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_culto]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Usage report'));
    await tester.pumpAndSettle();

    expect(find.byType(UsoPage), findsOneWidget);
  });

  testWidgets('repetir a agenda anterior manda o id do nome e do relógio', (
    tester,
  ) async {
    var carregamentos = 0;
    String? basePedida;
    String? idPedido;
    String? nomePedido;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) async {
            carregamentos++;
            return carregamentos == 1
                ? const <EventoAgenda>[_culto, _ensaio]
                : const <EventoAgenda>[_culto, _ensaio, repetido];
          },
          repetir:
              ({
                required String baseDir,
                required String novoId,
                required String nome,
                String? data,
              }) async {
                basePedida = baseDir;
                idPedido = novoId;
                nomePedido = nome;
                return repetido;
              },
          relogio: () => DateTime.utc(2026, 11, 29, 20, 0, 0),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Repeat the previous event'));
    await tester.pumpAndSettle();

    // O anterior é o último evento da lista — a mesma ordem do núcleo.
    expect(nomePedido, 'Ensaio de quinta');
    expect(idPedido, 'ensaio-de-quinta-20261129-200000');
    expect(basePedida, '/tmp/biblioteca');
    expect(carregamentos, 2, reason: 'a lista recarrega depois de repetir');
    expect(
      find.text('Event copied from the previous one: check the versions.'),
      findsOneWidget,
    );
  });

  testWidgets('repetir sem evento anterior avisa em vez de chamar o núcleo', (
    tester,
  ) async {
    var chamou = false;
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[]),
          repetir:
              ({
                required String baseDir,
                required String novoId,
                required String nome,
                String? data,
              }) async {
                chamou = true;
                return repetido;
              },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Repeat the previous event'));
    await tester.pumpAndSettle();

    expect(find.text('There is no previous event to repeat.'), findsOneWidget);
    expect(chamou, isFalse, reason: 'sem anterior não há o que repetir');
  });

  testWidgets('a recusa do núcleo ao repetir aparece sem esvaziar a lista', (
    tester,
  ) async {
    await tester.pumpWidget(
      _montar(
        AgendaPage(
          diretorioBase: '/tmp/biblioteca',
          carregarAgenda: (String _) => _agenda(const <EventoAgenda>[_culto]),
          repetir:
              ({
                required String baseDir,
                required String novoId,
                required String nome,
                String? data,
              }) async => throw StateError(
                'já existe um evento com o id culto-da-manha-20261129: repetir não sobrescreve',
              ),
          relogio: () => DateTime.utc(2026, 11, 29),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Repeat the previous event'));
    await tester.pumpAndSettle();

    expect(
      find.textContaining('Could not repeat the previous event'),
      findsOneWidget,
    );
    expect(
      find.textContaining('já existe um evento com o id'),
      findsOneWidget,
      reason: 'a mensagem do núcleo vem inteira',
    );
    // Falha de ação não é agenda vazia: a lista continua na tela.
    expect(find.text('Culto da manhã'), findsOneWidget);
    expect(find.text('No events in the agenda yet.'), findsNothing);
  });
}
