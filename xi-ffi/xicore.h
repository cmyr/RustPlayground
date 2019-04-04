#ifndef XI_CORE_H
#define XI_CORE_H
typedef struct _XiCore XiCore;

typedef struct _XiLine {
    char *text;
    int32_t cursor;
    int32_t selection[2];
} XiLine;

typedef const char* json;

typedef struct _XiSize {
    size_t width;
    size_t height;
} XiSize;

typedef void (*rpc_callback)(json);
typedef XiSize (*width_measure_fn)(const char*);
typedef void (*invalidate_callback)(size_t start, size_t end);

extern XiCore* xiCoreCreate(rpc_callback, invalidate_callback, width_measure_fn);
extern void xiCoreFree(XiCore*);
extern void xiCoreSendMessage(XiCore*, json);
extern void xiCStringFree(char*);
extern XiLine* xiCoreGetLine(XiCore*, uint32_t);
extern void xiLineFree(XiLine*);

#endif
