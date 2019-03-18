#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct _XiEventHandler XiEventHandler;
typedef void (*callback)(uint32_t);

extern XiEventHandler* xiEventHandlerCreate(callback);
extern void xiEventHandlerFree(XiEventHandler*);
extern int32_t xiEventHandlerHandleInput(const XiEventHandler*, uint32_t);
