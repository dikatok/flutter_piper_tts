import 'dart:io';

import 'package:ffigen/ffigen.dart';

void main() {
  final packageRoot = Platform.script.resolve('../');
  FfiGenerator(
    headers: Headers(entryPoints: [packageRoot.resolve('rust/bindings.h')]),
    output: Output(dartFile: packageRoot.resolve('lib/ffi.g.dart')),
    functions: Functions.includeSet({"init", 'create_instance', 'speak'}),
    structs: Structs.includeSet({'FFIReponse'}),
  ).generate();
}
