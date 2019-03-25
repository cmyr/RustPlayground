#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

typedef struct _XiEventHandler XiEventHandler;
typedef struct _XiEventPayload XiEventPayload;

typedef uint32_t xi_millis;
typedef uint32_t xi_timer_token;

typedef void (*event_callback)(const XiEventPayload*, bool);
typedef void (*action_callback)(const char*);
typedef xi_timer_token (*timer_callback)(const XiEventPayload*, xi_millis);
typedef void (*cancel_timer_callback)(xi_timer_token);

extern XiEventHandler* xiEventHandlerCreate(event_callback, action_callback, timer_callback, cancel_timer_callback);
extern void xiEventHandlerFree(XiEventHandler*);
extern void xiEventHandlerHandleInput(const XiEventHandler*, uint32_t, const char*, XiEventPayload*);
