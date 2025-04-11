#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

const char *get_apdu(void);

void set_apdu(const char *apdu);

const char *get_apdu_return(void);

void set_apdu_return(const char *apdu_return);

void set_callback(const char *(*callback)(const char *apdu, int32_t timeout));

void imkey_free_const_string(const char *s);

/**
 * dispatch protobuf rpc call
 */
const char *call_imkey_api(const char *hex_str);

void imkey_clear_err(void);

const char *imkey_get_last_err_message(void);
