import 'package:flutter/material.dart';
import 'package:core_l10n/core_l10n.dart';

void main() {
  runApp(const KentronApp());
}

class KentronApp extends StatelessWidget {
  const KentronApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Kentron Music Hub',
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.emerald),
      ),
      home: const HomeScreen(),
    );
  }
}

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Kentron')),
      body: const Center(
        child: Text('Welcome to Kentron Music Hub'),
      ),
    );
  }
}
