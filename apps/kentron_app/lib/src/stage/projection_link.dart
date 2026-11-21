import 'dart:convert';

/// Parâmetro de URL que liga o modo projeção.
///
/// O Browser Source do OBS aponta para uma URL com a própria cifra:
/// `?projecao=<base64url do ChordPro>`. Base64url evita `&`, `#` e acentos
/// soltos no caminho do navegador — o texto vira um token opaco.
const projectionQueryParam = 'projecao';

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
/// que o operador cola no Browser Source do OBS.
Uri urlDaProjecao(String chordpro, {Uri? base}) {
  final raiz = base ?? Uri.base;
  return raiz.replace(
    queryParameters: <String, String>{
      projectionQueryParam: base64Url.encode(utf8.encode(chordpro)),
    },
  );
}
