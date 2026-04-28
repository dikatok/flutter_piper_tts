import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter_piper_tts/ffi.g.dart' as g;

class PiperTTS {
  final int fd;

  PiperTTS._(this.fd);

  static void init() => g.init();

  static PiperTTS create(String modelPath, String configPath) {
    final modelPathPointer = modelPath.toNativeUtf8();
    final configPathPointer = configPath.toNativeUtf8();

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
      final result = g.speak(fd, textPointer.cast<Char>());
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
    }
  }
}

extension on Pointer<Char> {
  bool get isEmpty => this == nullptr || cast<Utf8>().toDartString().isEmpty;

  bool get isNotEmpty => !isEmpty;

  String toDartString() => cast<Utf8>().toDartString();
}
