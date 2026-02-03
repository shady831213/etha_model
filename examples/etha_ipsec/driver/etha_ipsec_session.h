#ifndef __ETHA_IPSEC_SESSION_H__
#define __ETHA_IPSEC_SESSION_H__
#include <stdint.h>
typedef enum
{
    CipherAlgNull = 0,
    AES128 = 1,
    AES256 = 2,
} CipherAlg;

typedef enum
{
    CBC = 0,
    CCM = 1,
    GCM = 2,
} CipherMode;

typedef enum
{
    HmacAlgNull = 0,
    SHA1 = 1,
    SHA256 = 2,
    SHA512 = 3,
} HmacAlg;

typedef struct
{
    uint32_t id;
} EthaIpsecSession;

void etha_ipsec_session_init(EthaIpsecSession *self, uint32_t id);
void etha_ipsec_session_cfg(EthaIpsecSession *self, const uint8_t *cipher_key, uint32_t salt, const uint8_t *hash_key, uint32_t salt_len, uint32_t iv_len, CipherAlg cipher_alg, CipherMode cipher_mode, HmacAlg hmac_alg);
void etha_ipsec_session_enable(EthaIpsecSession *self);
void etha_ipsec_session_disable(EthaIpsecSession *self);

#endif