import 'package:core_l10n/core_l10n.dart';
import 'package:material_ui/material_ui.dart';

/// A falha do núcleo **na tela**: o que a operação era e a mensagem que veio dele.
///
/// Existe como widget porque a agenda, o editor e o relatório dizem a mesma
/// coisa de formas diferentes — e o que não pode variar é o erro aparecer. Erro
/// do núcleo nunca vira lista vazia nem valor inventado (fail-loud).
class AreaDeErro extends StatelessWidget {
  const AreaDeErro({
    super.key,
    required this.mensagem,
    required this.erro,
    this.onTentarNovamente,
  });

  /// O que falhou, já localizado (ex.: `agendaLoadFailed`).
  final String mensagem;

  /// A mensagem do núcleo, como ela veio — sem tradução e sem resumo.
  final String erro;

  /// Recarrega, quando a tela tem um caminho de recarga. Sem ele, o botão não
  /// aparece: botão que não faz nada é pior que botão ausente.
  final VoidCallback? onTentarNovamente;

  @override
  Widget build(BuildContext context) {
    final CoreL10n l10n = CoreL10n.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          Icon(
            Icons.error_outline,
            size: 48,
            color: Theme.of(context).colorScheme.error,
          ),
          const SizedBox(height: 12),
          Text(mensagem, textAlign: TextAlign.center),
          const SizedBox(height: 8),
          Text(
            erro,
            style: const TextStyle(fontSize: 12, color: Colors.grey),
            textAlign: TextAlign.center,
          ),
          if (onTentarNovamente != null) ...<Widget>[
            const SizedBox(height: 16),
            FilledButton(
              onPressed: onTentarNovamente,
              child: Text(l10n.agendaRetry),
            ),
          ],
        ],
      ),
    );
  }
}
