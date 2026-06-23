import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_piper_tts/flutter_piper_tts.dart';
import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';

const modelFile = 'ne_NP-google-medium.onnx';
const configFile = 'ne_NP-google-medium.onnx.json';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(MaterialApp(home: Initial()));
}

class Initial extends StatelessWidget {
  const Initial({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            FilledButton(
              onPressed: () => Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => MyApp()),
              ),
              child: const Text('Start'),
            ),
          ],
        ),
      ),
    );
  }
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final controller = TextEditingController();

  late PiperTTS tts;

  @override
  void initState() {
    super.initState();
    unawaited(() async {
      final (modelPath, configPath) = await copyModelFromAssets();
      tts = await PiperTTS.create(modelPath: modelPath, configPath: configPath);
      await tts.speak('Hello world!');
    }());
  }

  @override
  void dispose() async {
    super.dispose();
    await tts.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const textStyle = TextStyle(fontSize: 25);
    return Scaffold(
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
                  await tts.speak(
                    controller.text,
                    phonemizerStrategy: PhonemizerStrategy.neuralSentence,
                  );
                },
                child: const Text(
                  "speak (phonemizer: neuralSentence)",
                  style: textStyle,
                ),
              ),
              TextButton(
                onPressed: () async {
                  await tts.speak(
                    controller.text,
                    phonemizerStrategy: PhonemizerStrategy.neuralWord,
                  );
                },
                child: const Text(
                  "speak (phonemizer: neuralWord)",
                  style: textStyle,
                ),
              ),
              TextButton(
                onPressed: () async {
                  await tts.speak(
                    controller.text,
                    phonemizerStrategy:
                        PhonemizerStrategy.dictionaryWithNeuralFallback,
                  );
                },
                child: const Text(
                  "speak (phonemizer: dictionaryWithNeuralFallback)",
                  style: textStyle,
                ),
              ),
              TextButton(
                onPressed: () async {
                  await tts.speak(
                    controller.text,
                    phonemizerStrategy:
                        PhonemizerStrategy.dictionaryWithOmitUnknown,
                  );
                },
                child: const Text(
                  "speak (phonemizer: dictionaryWithOmitUnknown)",
                  style: textStyle,
                ),
              ),
              TextButton(
                onPressed: () async {
                  await tts.speakFromPhonemes(
                    phonemes: controller.text,
                    phonemeChunkSize: 0,
                  );
                },
                child: const Text("speak phonemes", style: textStyle),
              ),
              TextButton(
                onPressed: () async {
                  await tts.pause();
                },
                child: const Text("pause", style: textStyle),
              ),
              TextButton(
                onPressed: () async {
                  await tts.resume();
                },
                child: const Text("resume", style: textStyle),
              ),
              TextButton(
                onPressed: () async {
                  await tts.stop();
                },
                child: const Text("stop", style: textStyle),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

Future<(String, String)> copyModelFromAssets() async {
  final directory = await getApplicationSupportDirectory();
  final modelPath = join(directory.path, modelFile);
  final configPath = join(directory.path, configFile);

  final exists = await File(modelPath).exists();

  if (exists) {
    return (modelPath, configPath);
  }

  final modelData = await rootBundle.load("assets/$modelFile");
  List<int> bytes = modelData.buffer.asUint8List(
    modelData.offsetInBytes,
    modelData.lengthInBytes,
  );

  await File(modelPath).writeAsBytes(bytes, flush: true);

  final configData = await rootBundle.load("assets/$configFile");
  bytes = configData.buffer.asUint8List(
    configData.offsetInBytes,
    configData.lengthInBytes,
  );
  await File(configPath).writeAsBytes(bytes, flush: true);

  return (modelPath, configPath);
}
