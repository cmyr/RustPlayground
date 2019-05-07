#ifndef PLAYGROUND_H
#define PLAYGROUND_H

typedef const char* json;
typedef void (*stderr_callback)(const char*);

typedef struct _ExternError {
    int32_t code;
    char *message; // note: nullable
} ExternError;

extern json playgroundGetToolchains(ExternError* error);
extern json playgroundExecuteTask(const char* path, json, stderr_callback, ExternError* error);

extern void playgroundStringFree(json);

#endif
