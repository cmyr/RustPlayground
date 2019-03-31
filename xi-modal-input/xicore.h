#ifndef XI_CORE_H
#define XI_CORE_H
typedef struct _XiCore XiCore;

typedef struct _XiLine {
    char *text;
    int32_t cursor;
} XiLine;

typedef const char* json;

typedef void (*rpc_callback)(json);
typedef void (*update_callback)(uint32_t);

extern XiCore* xiCoreCreate(rpc_callback, update_callback);
extern void xiCoreFree(XiCore*);
extern void xiCoreSendMessage(XiCore*, json);
extern XiLine* xiCoreGetLine(XiCore*, uint32_t);

#endif
