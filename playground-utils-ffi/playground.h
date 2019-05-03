#ifndef PLAYGROUND_H
#define PLAYGROUND_H

typedef const char* json;
typedef void (*stderr_callback)(const char*);

extern json playgroundGetToolchains();
extern json playgroundExecuteTask(const char* path, json, stderr_callback);

extern void playgroundStringFree(json);

#endif
