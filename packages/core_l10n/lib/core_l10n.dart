/// Ponto único de internacionalização do Kentron.
///
/// Os ARBs, o `l10n.yaml` e a configuração de locale vivem neste pacote; o app
/// importa só `package:core_l10n/core_l10n.dart` e monta o `MaterialApp` com o
/// que vier daqui — nenhuma string de interface nem lista de locale é escrita
/// no app.
///
/// O arquivo gerado (`l10n/app_localizations.dart`) **não é versionado**: é
/// reproduzível a partir dos ARBs, e o pipeline roda `flutter gen-l10n` antes
/// de analisar e testar. Esta reexportação existe para que o app não precise
/// conhecer o caminho do gerado — se ele mudar, muda só aqui.
///
/// Uso (o app compõe os delegates com o pacote de UI dele):
/// ```dart
/// MaterialApp(
///   localizationsDelegates: const [
///     CoreL10n.delegate,
///     // GlobalMaterialLocalizations/GlobalWidgetsLocalizations vêm do pacote
///     // de UI do app (material_ui), não do flutter_localizations que o
///     // gerador emite — são tipos distintos.
///     GlobalMaterialLocalizations.delegate,
///     GlobalWidgetsLocalizations.delegate,
///   ],
///   supportedLocales: CoreL10n.supportedLocales,
///   home: ...,
/// )
/// ```
library;

export 'l10n/app_localizations.dart';
