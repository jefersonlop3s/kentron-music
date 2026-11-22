import 'dart:convert';

/// Parâmetro de URL que liga o modo projeção.
///
/// O Browser Source do OBS aponta para uma URL com a própria cifra:
/// `?projecao=<base64url do ChordPro>`. Base64url evita `&`, `#` e acentos
/// soltos no caminho do navegador — o texto vira um token opaco.
const projectionQueryParam = 'projecao';

/// Parâmetro de URL que escolhe a seção inicial da projeção.
///
/// O Browser Source do OBS **não recebe clique nem teclado** — não há canal
/// scriptável para o operador avançar a seção de fora do app. A URL é o único
/// canal determinístico: `?secao=2` abre a projeção na terceira seção (índice
/// zero-based), e o piloto automatizado da ADR-0010 navega trocando a URL.
/// O mesmo link por seção serve ao operador, que pode salvar um por estrofe.
const sectionQueryParam = 'secao';

/// Lê a seção inicial da URL, quando existe.
///
/// Devolve `null` quando o parâmetro está ausente, não é numérico ou é
/// negativo. A projeção limita o índice ao intervalo válido de qualquer forma;
/// aqui a recusa é só para não confundir "sem seção" com "seção zero".
int? secaoDaUrl(Uri url) {
  final bruto = url.queryParameters[sectionQueryParam];
  if (bruto == null || bruto.isEmpty) return null;
  final indice = int.tryParse(bruto);
  if (indice == null || indice < 0) return null;
  return indice;
}

/// Lê a cifra embutida na URL, quando existe.
///
/// Devolve `null` quando o parâmetro não existe ou não decodifica. URL
/// estragada não vira projeção vazia: o app decide o que mostrar.
String? chordproDaUrl(Uri url) {
  final bruto = url.queryParameters[projectionQueryParam];
  if (bruto == null || bruto.isEmpty) return null;
  try {
    return utf8.decode(base64Url.decode(base64Url.normalize(bruto)));
  } on FormatException {
    return null;
  }
}

/// Monta a URL de projeção a partir da base atual e da cifra.
///
/// Preserva host, caminho e fragmento; troca só a query. A URL resultante é a
/// que o operador cola no Browser Source do OBS. [secao] entra apenas quando
/// informada: sem ela a projeção começa na primeira seção.
Uri urlDaProjecao(String chordpro, {Uri? base, int? secao}) {
  final raiz = base ?? Uri.base;
  return raiz.replace(
    queryParameters: <String, String>{
      projectionQueryParam: base64Url.encode(utf8.encode(chordpro)),
      if (secao != null) sectionQueryParam: '$secao',
    },
  );
}

/// A recusa do portão de publicação do motor.
///
/// O motor responde `podePublicar`; quando ele bloqueia, a tela não gera link.
/// A exceção carrega o motivo que o motor deu, para a interface mostrá-lo.
class PublicacaoBloqueada implements Exception {
  const PublicacaoBloqueada(this.motivo);

  final String motivo;

  @override
  String toString() => motivo;
}

/// Monta o link de projeção **depois** do portão do motor.
///
/// `portao` é a função da ponte (`podePublicar`): ela lança quando a letra não
/// pode circular. O app não copia a regra de licença — pergunta ao motor e
/// obedece. Sem portão aprovado não há link: é o que faz o escopo privado do
/// grupo valer também no app, e não só no motor.
///
/// Lança [PublicacaoBloqueada] com o motivo do motor quando a publicação é
/// recusada.
Future<Uri> linkDeProjecao(
  String chordpro, {
  required Future<void> Function(String) portao,
  Uri? base,
  int? secao,
}) async {
  try {
    await portao(chordpro);
  } on Object catch (erro) {
    throw PublicacaoBloqueada(erro.toString());
  }
  return urlDaProjecao(chordpro, base: base, secao: secao);
}
