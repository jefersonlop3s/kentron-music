import 'package:flutter_test/flutter_test.dart';
import 'package:kentron_app/src/stage/projection_link.dart';

void main() {
  group('chordproDaUrl', () {
    test('devolve null sem o parâmetro', () {
      expect(chordproDaUrl(Uri.parse('https://exemplo.test/app/')), isNull);
    });

    test('devolve null quando o base64 não decodifica', () {
      expect(
        chordproDaUrl(Uri.parse('https://exemplo.test/app/?projecao=_w')),
        isNull,
      );
    });

    test('faz round-trip com acentos e símbolos', () {
      const cifra = '{title: Canção}\n{key: C}\n\n[Verse]\n[C]Santo & santo\n';
      final url = urlDaProjecao(
        cifra,
        base: Uri.parse('https://exemplo.test/app/'),
      );
      expect(chordproDaUrl(url), cifra);
    });
  });

  group('urlDaProjecao', () {
    test('preserva host, caminho e fragmento', () {
      final url = urlDaProjecao(
        '{title: X}',
        base: Uri.parse('https://exemplo.test/app/#/tela'),
      );
      expect(url.host, 'exemplo.test');
      expect(url.path, '/app/');
      expect(url.fragment, '/tela');
      expect(url.queryParameters, contains(projectionQueryParam));
    });

    test('troca a query existente', () {
      final url = urlDaProjecao(
        '{title: X}',
        base: Uri.parse('https://exemplo.test/app/?outro=1'),
      );
      expect(url.queryParameters.keys, <String>[projectionQueryParam]);
    });
  });

  group('linkDeProjecao', () {
    test('motor libera → monta o link', () async {
      final link = await linkDeProjecao(
        '{title: Livre}',
        portao: (_) async {},
        base: Uri.parse('https://exemplo.test/app/'),
      );
      expect(link.queryParameters, contains(projectionQueryParam));
      expect(chordproDaUrl(link), '{title: Livre}');
    });

    test('motor bloqueia → lança PublicacaoBloqueada e não monta link', () async {
      await expectLater(
        linkDeProjecao(
          '{title: Privada}',
          portao: (_) async => throw StateError('escopo privado do grupo'),
        ),
        throwsA(
          isA<PublicacaoBloqueada>().having(
            (bloqueio) => bloqueio.motivo,
            'motivo',
            contains('privado'),
          ),
        ),
      );
    });
  });
}
