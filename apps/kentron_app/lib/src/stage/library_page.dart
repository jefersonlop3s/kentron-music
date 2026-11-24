import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:kentron_app/src/rust/bridge.dart';
import 'package:kentron_app/src/stage/song_reader_page.dart';
import 'package:material_ui/material_ui.dart';

/// Diretório da biblioteca local no sistema de arquivos.
String _diretorioBase() {
  if (kIsWeb) return '/tmp/kentron-library';
  return '${Platform.environment['HOME'] ?? '/tmp'}/.kentron/library';
}

/// Tela da biblioteca local: lista as cifras salvas e permite abri-las.
class LibraryPage extends StatefulWidget {
  const LibraryPage({super.key});

  @override
  State<LibraryPage> createState() => _LibraryPageState();
}

class _LibraryPageState extends State<LibraryPage> {
  List<SongSummary> _musicas = <SongSummary>[];
  bool _carregando = true;
  String? _erro;

  @override
  void initState() {
    super.initState();
    _recarregar();
  }

  Future<void> _recarregar() async {
    setState(() { _carregando = true; _erro = null; });
    try {
      final musicas = await listarBiblioteca(baseDir: _diretorioBase());
      if (!mounted) return;
      setState(() { _musicas = musicas; _carregando = false; });
    } on Object catch (e) {
      if (!mounted) return;
      setState(() { _musicas = <SongSummary>[]; _carregando = false; _erro = e.toString(); });
    }
  }

  Future<void> _adicionarCifra() async {
    final chordpro = await _mostrarDialogoDeEntrada();
    if (chordpro == null || chordpro.trim().isEmpty) return;
    try {
      final resumo = await parseSong(input: chordpro);
      final id = resumo.title.isNotEmpty
          ? resumo.title.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '-')
          : 'cifra-${DateTime.now().millisecondsSinceEpoch}';
      await salvarNaBiblioteca(baseDir: _diretorioBase(), id: id, chordpro: chordpro);
      await _recarregar();
    } on Object catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Erro: $e')));
    }
  }

  Future<String?> _mostrarDialogoDeEntrada() {
    final controller = TextEditingController();
    return showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Adicionar cifra'),
        content: SizedBox(
          width: double.maxFinite,
          child: TextField(
            controller: controller,
            maxLines: 10,
            decoration: const InputDecoration(
              hintText: 'Cole o ChordPro aqui...\n{title: Nome}\n{key: C}\n\n[Verse]\n[C]Letra...',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        actions: <Widget>[
          TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancelar')),
          FilledButton(onPressed: () => Navigator.of(context).pop(controller.text), child: const Text('Salvar')),
        ],
      ),
    );
  }

  Future<void> _removerCifra(String id) async {
    try {
      await removerDaBiblioteca(baseDir: _diretorioBase(), id: id);
      await _recarregar();
    } on Object catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Erro: $e')));
    }
  }

  void _abrirModoPalco(int indice) {
    if (indice >= _musicas.length) return;
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => SongReaderPage(
          input: '',
          biblioteca: _musicas,
          indiceNaBiblioteca: indice,
          diretorioBase: _diretorioBase(),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Biblioteca'),
        actions: <Widget>[
          IconButton(tooltip: 'Adicionar cifra', onPressed: _adicionarCifra, icon: const Icon(Icons.add)),
          IconButton(tooltip: 'Recarregar', onPressed: _recarregar, icon: const Icon(Icons.refresh)),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_carregando) return const Center(child: CircularProgressIndicator());

    if (_erro != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const Icon(Icons.error_outline, size: 48),
              const SizedBox(height: 12),
              const Text('Não foi possível carregar a biblioteca.', textAlign: TextAlign.center),
              const SizedBox(height: 8),
              Text(_erro!, style: const TextStyle(fontSize: 12, color: Colors.grey), textAlign: TextAlign.center),
              const SizedBox(height: 16),
              FilledButton(onPressed: _recarregar, child: const Text('Tentar novamente')),
            ],
          ),
        ),
      );
    }

    if (_musicas.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const Icon(Icons.library_music, size: 64),
              const SizedBox(height: 16),
              Text('Nenhuma cifra na biblioteca.', style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              const Text('Adicione cifras colando texto ChordPro.', textAlign: TextAlign.center),
              const SizedBox(height: 24),
              FilledButton.icon(onPressed: _adicionarCifra, icon: const Icon(Icons.add), label: const Text('Adicionar cifra')),
            ],
          ),
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _recarregar,
      child: ListView.builder(
        itemCount: _musicas.length,
        itemBuilder: (context, indice) {
          final musica = _musicas[indice];
          final tom = musica.key ?? '?';
          return ListTile(
            leading: const Icon(Icons.music_note),
            title: Text(musica.title.isNotEmpty ? musica.title : '(sem título)'),
            subtitle: Text('${musica.artist ?? 'sem artista'} · tom $tom · ${musica.sections} seção(ões)'),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                IconButton(icon: const Icon(Icons.visibility), tooltip: 'Abrir no Modo Palco', onPressed: () => _abrirModoPalco(indice)),
                IconButton(icon: const Icon(Icons.delete_outline), tooltip: 'Remover', onPressed: () {
                  final id = musica.title.isNotEmpty
                      ? musica.title.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '-')
                      : '';
                  _removerCifra(id);
                }),
              ],
            ),
          );
        },
      ),
    );
  }
}

/// Lê uma cifra da biblioteca pelo índice na lista.
Future<String> carregarCifraDaBiblioteca({required String baseDir, required String id}) =>
    carregarDaBiblioteca(baseDir: baseDir, id: id);