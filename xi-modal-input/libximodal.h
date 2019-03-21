#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct _XiEventHandler XiEventHandler;
typedef struct _XiEventPayload XiEventPayload;
typedef void (*callback)(const XiEventPayload*);

extern XiEventHandler* xiEventHandlerCreate(callback);
extern void xiEventHandlerFree(XiEventHandler*);
extern void xiEventHandlerHandleInput(const XiEventHandler*, uint32_t, const char*, XiEventPayload*);
