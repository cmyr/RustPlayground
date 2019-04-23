#ifndef PLAYGROUND_H
#define PLAYGROUND_H

typedef const char* json;

extern json playgroundGetToolchains();
extern json playgroundExecuteTask(const char* path, json);

extern void playgroundStringFree(json);

#endif
