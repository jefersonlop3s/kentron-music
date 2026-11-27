import 'dart:async';

import 'package:core_l10n/core_l10n.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/library_page.dart';
import 'package:kentron_app/src/stage/uso_page.dart';
import 'package:material_ui/material_ui.dart';

/// O relatório de mentira, no formato que o núcleo devolve. O teste não toca em
/// FFI; a prova com o cdylib real vive em `bridge_test.dart`.
const relatorioComUso = RelatorioDeUso(
  eventos: 2,
  execucoes: 3,
  obrasDistintas: 2,
  publicas: 1,
  semFicha: 1,
  obras: <ObraDoRelatorio>[
    ObraDoRelatorio(
      titulo: 'Santo',
      execucoes: 2,
      versoes: <VersaoDaObra>[
        VersaoDaObra(versao: 'versão do coral', vezes: 1),
        VersaoDaObra(versao: '(versão não declarada)', vezes: 1),
      ],
      veredito: 'domínio público (ficha do hinário: domínio público desde 2018)',
      publica: true,
    ),
    ObraDoRelatorio(
      titulo: 'Hino 3',
      execucoes: 1,
      versoes: <VersaoDaObra>[
        VersaoDaObra(versao: '(versão não declarada)', vezes: 1),
      ],
      publica: false,
    ),
  ],
  core: 'falso 0.0.0',
);

const relatorioSemUso = RelatorioDeUso(
  eventos: 1,
  execucoes: 0,
  obrasDistintas: 0,
  publicas: 0,
  semFicha: 0,
  obras: <ObraDoRelatorio>[],
  core: 'falso 0.0.0',
);

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
    final resposta = Completer<RelatorioDeUso>();
    await tester.pumpWidget(
      _montar(
        UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String _, int __) => resposta.future,
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pump();

    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    resposta.complete(relatorioComUso);
    await tester.pumpAndSettle();
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('pede o relatório ao núcleo com o diretório e o ano do relógio', (
    tester,
  ) async {
    String? diretorioPedido;
    int? anoPedido;
    await tester.pumpWidget(
      _montar(
        UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String baseDir, int anoCorrente) async {
            diretorioPedido = baseDir;
            anoPedido = anoCorrente;
            return relatorioComUso;
          },
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(diretorioPedido, '/tmp/biblioteca');
    expect(anoPedido, 2026, reason: 'o prazo de domínio público conta do ano');
  });

  testWidgets('sem diretório informado, o relatório usa o diretório do app', (
    tester,
  ) async {
    String? diretorioPedido;
    await tester.pumpWidget(
      _montar(
        UsoPage(
          carregar: (String baseDir, int _) async {
            diretorioPedido = baseDir;
            return relatorioComUso;
          },
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(diretorioPedido, bibliotecaDir());
    expect(diretorioPedido, isNot(''));
  });

  testWidgets('lista o uso, as versões e o veredito de cada obra', (tester) async {
    await tester.pumpWidget(
      _montar(
        UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String _, int __) async => relatorioComUso,
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('2 events'), findsOneWidget);
    expect(find.text('3 executions'), findsOneWidget);
    expect(find.text('2 works'), findsOneWidget);
    expect(find.text('1 work in public domain'), findsOneWidget);
    expect(find.text('1 work without a record'), findsOneWidget);

    expect(find.text('Santo'), findsOneWidget);
    expect(find.text('2×'), findsOneWidget);
    expect(find.text('1× versão do coral'), findsOneWidget);
    expect(
      find.text('1× (versão não declarada)'),
      findsWidgets,
      reason: 'a versão não declarada é dita, não inventada',
    );
    expect(
      find.text(
        'domínio público (ficha do hinário: domínio público desde 2018)',
      ),
      findsOneWidget,
    );

    // `Hino 3` não tem ficha: a tela diz isso e não afirma nada sobre licença.
    expect(find.text('Hino 3'), findsOneWidget);
    expect(find.text('no record'), findsOneWidget);
    expect(find.text('núcleo: falso 0.0.0'), findsOneWidget);
  });

  testWidgets('agenda sem execução é o estado vazio, e diz por quê', (tester) async {
    await tester.pumpWidget(
      _montar(
        UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String _, int __) async => relatorioSemUso,
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(
      find.text('No song was sung yet: the agenda has no execution.'),
      findsOneWidget,
    );
    expect(find.text('no record'), findsNothing);
  });

  testWidgets('o erro do núcleo aparece em vez de virar relatório vazio', (
    tester,
  ) async {
    await tester.pumpWidget(
      _montar(
        UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String _, int __) async =>
              throw StateError('missing file: library.json'),
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Could not load the usage report'), findsOneWidget);
    expect(find.textContaining('missing file: library.json'), findsOneWidget);
    // O vazio não pode aparecer no lugar do erro.
    expect(
      find.text('No song was sung yet: the agenda has no execution.'),
      findsNothing,
    );
  });

  testWidgets('a falha do relatório em português também aparece na tela', (
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
        home: UsoPage(
          diretorioBase: '/tmp/biblioteca',
          carregar: (String _, int __) async =>
              throw StateError('biblioteca em "/tmp": missing file'),
          anoCorrente: () => 2026,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(
      find.text('Não foi possível carregar o relatório de uso'),
      findsOneWidget,
    );
    expect(find.textContaining('missing file'), findsOneWidget);
  });
}
