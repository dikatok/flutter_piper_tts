import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_piper_tts/flutter_piper_tts.dart';
import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final (modelPath, configPath) = await copyModelFromAssets();
  await PiperTTS.init();
  final tts = await PiperTTS.create(
    modelPath: modelPath,
    configPath: configPath,
  );
  tts.speak('Hello world!');
  runApp(MyApp(tts: tts));
}

class MyApp extends StatefulWidget {
  final PiperTTS tts;

  const MyApp({super.key, required this.tts});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final controller = TextEditingController();

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    const textStyle = TextStyle(fontSize: 25);
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Native Packages')),
        body: SingleChildScrollView(
          child: Container(
            padding: const .all(10),
            child: Column(
              spacing: 10,
              children: [
                TextField(
                  maxLines: null,
                  minLines: 1,
                  keyboardType: TextInputType.multiline,
                  decoration: InputDecoration(hintText: 'Type something...'),
                  controller: controller,
                ),
                TextButton(
                  onPressed: () async {
                    await widget.tts.speak(controller.text);
                  },
                  child: const Text("speak", style: textStyle),
                ),
                TextButton(
                  onPressed: () {
                    widget.tts.pause();
                  },
                  child: const Text("pause", style: textStyle),
                ),
                TextButton(
                  onPressed: () {
                    widget.tts.resume();
                  },
                  child: const Text("resume", style: textStyle),
                ),
                TextButton(
                  onPressed: () {
                    widget.tts.stop();
                  },
                  child: const Text("stop", style: textStyle),
                ),
                TextButton(
                  onPressed: () {
                    widget.tts.dispose();
                  },
                  child: const Text("dispose", style: textStyle),
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

  if (exists) {
    return (modelPath, configPath);
  }

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

  return (modelPath, configPath);
}
