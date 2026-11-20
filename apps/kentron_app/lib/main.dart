import 'package:flutter/material.dart';

/// Sovereign Emerald, token visual do DESIGN.md.
const _sovereignEmerald = Color(0xFF10B981);

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
        colorScheme: ColorScheme.fromSeed(seedColor: _sovereignEmerald),
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
