import 'package:kentron_app/src/rust/bridge.dart';

/// Monta a linha de acordes que fica acima da letra.
///
/// O núcleo devolve cada acorde com a coluna onde ele começa na letra, contada
/// em caracteres (`ChordPlacement::position`). Como a cifra é desenhada em
/// fonte monoespaçada, coluna de caractere é coluna de texto — desde que a
/// letra não tenha caracteres fora do BMP (emoji, por exemplo). Nesse caso o
/// alinhamento desloca; limite conhecido, não bug.
///
/// Acorde que invadiria o anterior é empurrado para a direita: acorde e letra
/// nunca se sobrescrevem, porque a letra é o que a banda canta.
String chordLine(List<ChordMark> marks) {
  final ordenados = <ChordMark>[...marks]
    ..sort((a, b) => a.position.compareTo(b.position));
  final buffer = StringBuffer();
  var coluna = 0;
  for (final mark in ordenados) {
    final alvo = mark.position < coluna ? coluna + 1 : mark.position;
    if (alvo > coluna) {
      buffer.write(''.padRight(alvo - coluna));
    }
    buffer.write(mark.chord);
    coluna = alvo + mark.chord.length;
  }
  return buffer.toString();
}
