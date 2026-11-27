import 'package:core_l10n/core_l10n.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/agenda_editor_page.dart';
import 'package:material_ui/material_ui.dart';

/// A biblioteca de mentira: as obras com o id e o tom que o núcleo declara hoje.
/// O teste não toca em FFI — a prova com o cdylib real vive em `bridge_test.dart`.
const obrasDaBiblioteca = <SongSummary>[
  SongSummary(
    id: 'santo',
    title: 'Santo',
    key: 'C',
    sections: 1,
    chords: <String>[],
    core: 'falso 0.0.0',
  ),
  SongSummary(
    id: 'arranjo-coral',
    title: 'Santo (arranjo do coral)',
    key: 'E',
    sections: 1,
    chords: <String>[],
    core: 'falso 0.0.0',
  ),
];

Future<List<SongSummary>> _biblioteca(String baseDir) async => obrasDaBiblioteca;

Future<List<String>> _tons() async => const <String>['C', 'D', 'E'];

/// Espião de gravação: guarda o que a tela mandou e devolve o evento como o
/// núcleo devolveria. Com [erro] informado, recusa como o núcleo recusa.
class SalvarEspiao {
  String? baseDir;
  String? id;
  String? nome;
  String? data;
  String? tipo;
  List<EntradaAgenda>? execucoes;
  Object? erro;
  int chamadas = 0;

  Future<EventoAgenda> chamar({
    required String baseDir,
    required String id,
    required String nome,
    String? data,
    String? tipo,
    required List<EntradaAgenda> execucoes,
  }) async {
    chamadas++;
    this.baseDir = baseDir;
    this.id = id;
    this.nome = nome;
    this.data = data;
    this.tipo = tipo;
    this.execucoes = execucoes;
    final Object? falha = erro;
    if (falha != null) throw falha;
    return EventoAgenda(
      id: id,
      nome: nome,
      data: data,
      tipo: tipo,
      execucoes: const <ExecucaoAgenda>[],
      core: 'falso 0.0.0',
    );
  }
}

/// Monta o editor **dentro** de uma rota: o editor volta com o evento salvo, e o
/// pop precisa de uma tela embaixo.
Widget _montar(Widget editor, {void Function(EventoAgenda?)? aoVoltar}) =>
    MaterialApp(
      locale: const Locale('en'),
      localizationsDelegates: const <LocalizationsDelegate<dynamic>>[
        CoreL10n.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: CoreL10n.supportedLocales,
      home: Builder(
        builder: (BuildContext context) => Scaffold(
          body: Center(
            child: TextButton(
              onPressed: () async {
                final resultado = await Navigator.of(context).push<EventoAgenda>(
                  MaterialPageRoute<EventoAgenda>(builder: (_) => editor),
                );
                aoVoltar?.call(resultado);
              },
              child: const Text('abrir editor'),
            ),
          ),
        ),
      ),
    );

/// O evento que a agenda já tem: usado para provar a edição (mesmo id) e o que
/// o seletor esconde.
const eventoNaAgenda = EventoAgenda(
  id: 'culto-1',
  nome: 'Culto da manhã',
  data: '2026-11-22',
  tipo: 'culto',
  execucoes: <ExecucaoAgenda>[
    ExecucaoAgenda(
      songId: 'santo',
      titulo: 'Santo',
      versao: 'versão do coral',
      tom: 'D',
    ),
  ],
  core: 'falso 0.0.0',
);

Future<void> _abrirEditor(WidgetTester tester, Widget editor, {void Function(EventoAgenda?)? aoVoltar}) async {
  await tester.pumpWidget(_montar(editor, aoVoltar: aoVoltar));
  await tester.pumpAndSettle();
  await tester.tap(find.text('abrir editor'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('o evento novo chega ao núcleo com nome, data, tipo e execuções', (
    tester,
  ) async {
    final espiao = SalvarEspiao();
    EventoAgenda? devolvido;
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        relogio: () => DateTime.utc(2026, 11, 27, 19, 30, 5),
        salvar: espiao.chamar,
      ),
      aoVoltar: (EventoAgenda? evento) => devolvido = evento,
    );

    await tester.enterText(
      find.widgetWithText(TextField, 'Event name'),
      'Culto da noite',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Date'),
      '2026-11-29',
    );
    await tester.tap(find.byType(DropdownButton<String>).first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Rehearsal').last);
    await tester.pumpAndSettle();

    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();
    expect(find.text('Pick the song'), findsOneWidget);
    await tester.tap(find.text('Santo'));
    await tester.pumpAndSettle();

    await tester.enterText(
      find.widgetWithText(TextField, 'Version'),
      'versão do coral',
    );
    await tester.tap(find.byType(DropdownButton<String>).last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('D').last);
    await tester.pumpAndSettle();

    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(espiao.chamadas, 1);
    expect(espiao.baseDir, '/tmp/biblioteca');
    // O id vem do nome e da data: o núcleo exige um id que sirva de arquivo.
    expect(espiao.id, 'culto-da-noite-2026-11-29');
    expect(espiao.nome, 'Culto da noite');
    expect(espiao.data, '2026-11-29');
    expect(espiao.tipo, 'ensaio');
    expect(espiao.execucoes, hasLength(1));
    expect(espiao.execucoes!.single.songId, 'santo');
    expect(espiao.execucoes!.single.versao, 'versão do coral');
    expect(espiao.execucoes!.single.tom, 'D');

    // O editor fechou devolvendo o evento: a agenda recarrega com o que foi salvo.
    expect(find.text('abrir editor'), findsOneWidget);
    expect(devolvido?.id, 'culto-da-noite-2026-11-29');
  });

  testWidgets('a execução começa no tom que a biblioteca declara hoje', (
    tester,
  ) async {
    final espiao = SalvarEspiao();
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        relogio: () => DateTime.utc(2026, 11, 27, 19, 30, 5),
        salvar: espiao.chamar,
      ),
    );

    await tester.enterText(
      find.widgetWithText(TextField, 'Event name'),
      'Ensaio de quinta',
    );
    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();
    // A obra está em `E` na biblioteca: é o tom que a execução já traz.
    await tester.tap(find.text('Santo (arranjo do coral)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(espiao.execucoes!.single.songId, 'arranjo-coral');
    expect(espiao.execucoes!.single.tom, 'E');
    // Sem data, o id leva o relógio: dois eventos de mesmo nome não se
    // sobrescrevem.
    expect(espiao.id, 'ensaio-de-quinta-20261127-193005');
    expect(espiao.data, isNull, reason: 'data não informada não é inventada');
  });

  testWidgets('sem nome a tela recusa antes de chamar o núcleo', (tester) async {
    final espiao = SalvarEspiao();
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: espiao.chamar,
      ),
    );

    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(find.text('Name the event before saving.'), findsOneWidget);
    expect(espiao.chamadas, 0, reason: 'nada foi ao núcleo');
  });

  testWidgets('data fora de AAAA-MM-DD é recusada com o formato na mensagem', (
    tester,
  ) async {
    final espiao = SalvarEspiao();
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: espiao.chamar,
      ),
    );

    await tester.enterText(
      find.widgetWithText(TextField, 'Event name'),
      'Culto da noite',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Date'),
      '29/11/2026',
    );
    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(find.text('The date must be written as YYYY-MM-DD.'), findsOneWidget);
    expect(espiao.chamadas, 0);
  });

  testWidgets('a recusa do núcleo aparece na tela e o formulário continua ali', (
    tester,
  ) async {
    final espiao = SalvarEspiao()..erro = StateError('obra usada na agenda: Santo');
    EventoAgenda? devolvido;
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: espiao.chamar,
      ),
      aoVoltar: (EventoAgenda? evento) => devolvido = evento,
    );

    await tester.enterText(
      find.widgetWithText(TextField, 'Event name'),
      'Culto da noite',
    );
    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(find.text('Could not save the event'), findsOneWidget);
    expect(
      find.textContaining('obra usada na agenda: Santo'),
      findsOneWidget,
      reason: 'a mensagem do núcleo vem inteira, sem resumo',
    );
    // O editor continua aberto com o que foi digitado: nada se perdeu.
    expect(find.text('abrir editor'), findsNothing);
    expect(find.text('Culto da noite'), findsOneWidget);
    // E nada foi dado como salvo.
    expect(devolvido, isNull);
  });

  testWidgets('editar grava no id do evento e não cria outro', (tester) async {
    final espiao = SalvarEspiao();
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        evento: eventoNaAgenda,
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        relogio: () => DateTime.utc(2026, 11, 27),
        salvar: espiao.chamar,
      ),
    );

    // O formulário chega preenchido com o que está na agenda.
    expect(find.text('Edit event'), findsOneWidget);
    expect(find.text('Culto da manhã'), findsOneWidget);
    expect(find.text('2026-11-22'), findsOneWidget);
    expect(find.text('Santo'), findsOneWidget);
    expect(find.text('versão do coral'), findsOneWidget);

    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(espiao.id, 'culto-1', reason: 'o id é o do evento: isto é edição');
    expect(espiao.nome, 'Culto da manhã');
    expect(espiao.data, '2026-11-22');
    expect(espiao.tipo, 'culto');
    expect(espiao.execucoes!.single.songId, 'santo');
    expect(espiao.execucoes!.single.versao, 'versão do coral');
    expect(espiao.execucoes!.single.tom, 'D');
  });

  testWidgets('remover a execução tira a obra do evento gravado', (tester) async {
    final espiao = SalvarEspiao();
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        evento: eventoNaAgenda,
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: espiao.chamar,
      ),
    );

    await tester.tap(find.byTooltip('Remove song'));
    await tester.pumpAndSettle();
    expect(find.text('No song in this event yet.'), findsOneWidget);

    await tester.tap(find.text('Save event'));
    await tester.pumpAndSettle();

    expect(espiao.execucoes, isEmpty, reason: 'o evento fica sem execução');
  });

  testWidgets('o seletor não oferece a obra que já está no evento', (tester) async {
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        evento: eventoNaAgenda,
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: SalvarEspiao().chamar,
      ),
    );

    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();

    // `santo` já está no evento: o núcleo recusaria a execução repetida, então a
    // tela não a oferece — mas a outra obra continua ali.
    expect(
      find.descendant(of: find.byType(AlertDialog), matching: find.text('Santo')),
      findsNothing,
    );
    expect(find.text('Santo (arranjo do coral)'), findsOneWidget);
  });

  testWidgets('biblioteca ilegível aparece no seletor, não como lista vazia', (
    tester,
  ) async {
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: (String _) async =>
            throw StateError('sem library.json'),
        tons: _tons,
        salvar: SalvarEspiao().chamar,
      ),
    );

    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();

    expect(find.text('Could not list the library'), findsOneWidget);
    expect(find.textContaining('sem library.json'), findsOneWidget);
    expect(find.text('The library has no chord chart yet.'), findsNothing);
  });

  testWidgets('filtro sem resultado não diz que a biblioteca está vazia', (
    tester,
  ) async {
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: _biblioteca,
        tons: _tons,
        salvar: SalvarEspiao().chamar,
      ),
    );

    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.widgetWithText(TextField, 'Filter by title'),
      'zzz',
    );
    await tester.pumpAndSettle();

    expect(find.text('No song with this title.'), findsOneWidget);
    expect(find.text('The library has no chord chart yet.'), findsNothing);
  });

  testWidgets('biblioteca vazia é o vazio do seletor', (tester) async {
    await _abrirEditor(
      tester,
      AgendaEditorPage(
        diretorioBase: '/tmp/biblioteca',
        carregarBiblioteca: (String _) async => const <SongSummary>[],
        tons: _tons,
        salvar: SalvarEspiao().chamar,
      ),
    );

    await tester.tap(find.text('Add song'));
    await tester.pumpAndSettle();

    expect(find.text('The library has no chord chart yet.'), findsOneWidget);
    expect(find.text('No song with this title.'), findsNothing);
  });
}
