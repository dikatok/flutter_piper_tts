#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct FFIInitResponse {
  char *error_message;
} FFIInitResponse;

typedef int64_t DartPort;

typedef void (*CompletionCallback)(DartPort port);

typedef struct FFICreateInstanceResponse {
  void *instance;
  char *error_message;
} FFICreateInstanceResponse;

typedef struct FFISpeakResponse {
  char *error_message;
} FFISpeakResponse;

typedef struct FFIPauseResponse {
  char *error_message;
} FFIPauseResponse;

typedef struct FFIResumeResponse {
  char *error_message;
} FFIResumeResponse;

typedef struct FFIStopResponse {
  char *error_message;
} FFIStopResponse;

struct FFIInitResponse init(CompletionCallback completion_cb, bool is_debug);

struct FFICreateInstanceResponse create_instance(const char *model_path, const char *config_path);

struct FFISpeakResponse speak(void *instance_ptr,
                              const char *text,
                              bool is_phonemes,
                              DartPort port,
                              const char *phonemization_strategy);

struct FFIPauseResponse pause(void *instance_ptr);

struct FFIResumeResponse resume(void *instance_ptr);

struct FFIStopResponse stop(void *instance_ptr);

void dispose(void *instance_ptr);
