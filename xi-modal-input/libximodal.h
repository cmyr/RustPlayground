#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct _XiEventHandler XiEventHandler;
typedef struct _XiEventPayload XiEventPayload;
typedef void (*callback)(const XiEventPayload*, bool);
typedef void (*action)(const char*);

extern XiEventHandler* xiEventHandlerCreate(callback, action);
extern void xiEventHandlerFree(XiEventHandler*);
extern void xiEventHandlerHandleInput(const XiEventHandler*, uint32_t, const char*, XiEventPayload*);
