import 'dart:ffi';

import 'package:dart_piper_tts/src/ffi.g.dart' as g;
import 'package:ffi/ffi.dart';

class PiperTTS {
  final int _fd;

  PiperTTS._(this._fd);

  static void init(({String dataDir}) args) {
    final dataDirPointer = args.dataDir.toNativeUtf8();
    try {
      final result = g.init(dataDirPointer.cast<Char>());
      final g.FFIInitResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(dataDirPointer);
    }
  }

  static PiperTTS create(({String modelPath, String configPath}) args) {
    final modelPathPointer = args.modelPath.toNativeUtf8();
    final configPathPointer = args.configPath.toNativeUtf8();

    try {
      final result = g.create_instance(
        modelPathPointer.cast<Char>(),
        configPathPointer.cast<Char>(),
      );
      final g.FFICreateInstanceResponse(:fd, :error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
      return PiperTTS._(fd);
    } finally {
      calloc.free(modelPathPointer);
      calloc.free(configPathPointer);
    }
  }

  void speak(String text) {
    final textPointer = text.toNativeUtf8();
    try {
      final result = g.speak(_fd, textPointer.cast<Char>());
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
    }
  }

  void pause([dynamic _]) {
    final result = g.pause(_fd);
    final g.FFIPauseResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void resume([dynamic _]) {
    final result = g.resume(_fd);
    final g.FFIResumeResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void stop([dynamic _]) {
    final result = g.stop(_fd);
    final g.FFIStopResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }
}

extension on Pointer<Char> {
  bool get isEmpty => this == nullptr || cast<Utf8>().toDartString().isEmpty;

  bool get isNotEmpty => !isEmpty;

  String toDartString() => cast<Utf8>().toDartString();
}
