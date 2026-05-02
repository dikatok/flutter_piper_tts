#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define BOS '^'

#define EOS '$'

#define PAD '_'

typedef struct FFIInitResponse {
  char *error_message;
} FFIInitResponse;

typedef struct FFICreateInstanceResponse {
  int32_t fd;
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

struct FFIInitResponse init(const char *data_dir);

struct FFICreateInstanceResponse create_instance(const char *model_path, const char *config_path);

struct FFISpeakResponse speak(int32_t fd, const char *text);

struct FFIPauseResponse pause(int32_t fd);

struct FFIResumeResponse resume(int32_t fd);

struct FFIStopResponse stop(int32_t fd);
