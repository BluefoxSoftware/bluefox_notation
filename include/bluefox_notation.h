#pragma once
#ifndef BLUEFOX_NOTATION

enum CBluefoxDataTypes {
    BLUEFOX_NULL,
    BLUEFOX_BOOL,
    BLUEFOX_INT,
    BLUEFOX_FLOAT,
    BLUEFOX_STRING,
    BLUEFOX_FUNCTION,
    BLUEFOX_ARRAY,
    BLUEFOX_DATA,
};

typedef struct CBluefoxArray {
    long l;
    void** d;
} CBluefoxArray;

extern CBluefoxArray bluefox_new_array();

typedef struct CBluefoxDataType {
    long t;
    void* v;
} CBluefoxDataType;

extern void bluefox_array_push(CBluefoxArray*, CBluefoxDataType);
extern CBluefoxDataType* bluefox_array_get(CBluefoxArray*, long);

extern CBluefoxDataType bluefox_new_null_data();
extern CBluefoxDataType bluefox_new_bool_data(long);
extern CBluefoxDataType bluefox_new_int_data(long);
extern CBluefoxDataType bluefox_new_float_data(double);
extern CBluefoxDataType bluefox_new_string_data(char*);
extern CBluefoxDataType bluefox_new_function_data(char*);
extern CBluefoxDataType bluefox_new_array_data(CBluefoxArray);

extern long bluefox_data_is_null(CBluefoxDataType*);
extern long* bluefox_data_get_bool(CBluefoxDataType*);
extern long* bluefox_data_get_int(CBluefoxDataType*);
extern double* bluefox_data_get_float(CBluefoxDataType*);
extern char* bluefox_data_get_string(CBluefoxDataType*);
extern char* bluefox_data_get_function(CBluefoxDataType*);
extern CBluefoxArray* bluefox_data_get_array(CBluefoxDataType*);

typedef struct CBluefoxData {
    long l;
    char** k;
    CBluefoxDataType* v;
} CBluefoxData;

extern CBluefoxData* bluefox_data_get_data(CBluefoxDataType*);
extern CBluefoxDataType bluefox_new_data_data(CBluefoxData);

extern CBluefoxData* bluefox_new_data();
extern void bluefox_data_insert(CBluefoxData*, char*, CBluefoxDataType);
extern CBluefoxDataType* bluefox_data_get(CBluefoxData*, char*);

// not stable, do not use
/*extern void bluefox_destroy_type(CBluefoxDataType*);
extern void bluefox_destroy_array(CBluefoxArray*);
extern void bluefox_destroy_data(CBluefoxData*);*/

#define BLUEFOX_NOTATION
#endif // BLUEFOX_NOTTAION