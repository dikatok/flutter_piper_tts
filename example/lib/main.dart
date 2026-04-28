import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_piper_tts/flutter_piper_tts.dart';
import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final (modelPath, configPath) = await copyModelFromAssets();
  PiperTTS.init();
  final tts = PiperTTS.create(modelPath, configPath);
  compute(tts.speak, "Hello World!");
  runApp(MyApp(tts: tts));
}

class MyApp extends StatefulWidget {
  final PiperTTS tts;

  const MyApp({super.key, required this.tts});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    const textStyle = TextStyle(fontSize: 25);
    const spacerSmall = SizedBox(height: 10);
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Native Packages')),
        body: SingleChildScrollView(
          child: Container(
            padding: const .all(10),
            child: Column(
              children: [
                const Text(
                  'This calls a native function through FFI that is shipped as source in the package. '
                  'The native code is built as part of the Flutter Runner build.',
                  style: textStyle,
                  textAlign: .center,
                ),
                spacerSmall,
                TextButton(
                  onPressed: () {
                    compute(widget.tts.speak, "Hello World!");
                  },
                  child: const Text("speak", style: textStyle),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

Future<(String, String)> copyModelFromAssets() async {
  final directory = await getApplicationSupportDirectory();
  final modelPath = join(directory.path, 'en_US-hfc_female-medium.onnx');
  final configPath = join(directory.path, 'en_US-hfc_female-medium.onnx.json');

  final exists = await File(modelPath).exists();

  if (!exists) {
    final modelData = await rootBundle.load(
      'assets/en_US-hfc_female-medium.onnx',
    );
    List<int> bytes = modelData.buffer.asUint8List(
      modelData.offsetInBytes,
      modelData.lengthInBytes,
    );

    await File(modelPath).writeAsBytes(bytes, flush: true);

    final configData = await rootBundle.load(
      'assets/en_US-hfc_female-medium.onnx.json',
    );
    bytes = configData.buffer.asUint8List(
      configData.offsetInBytes,
      configData.lengthInBytes,
    );
    await File(configPath).writeAsBytes(bytes, flush: true);
  }

  return (modelPath, configPath);
}
