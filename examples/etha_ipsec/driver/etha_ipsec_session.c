#ifndef __ETHA_IPSEC_SESSION_C__
#define __ETHA_IPSEC_SESSION_C__
#include <etha_ipsec_session.h>
#include <etha_ipsec_regs.h>
#include <etha_model.h>
#include <stdlib.h>
void etha_ipsec_session_init(EthaIpsecSession *self, uint32_t id)
{
    self->id = id;
}
void etha_ipsec_session_cfg(EthaIpsecSession *self, const uint8_t *cipher_key, uint32_t salt, const uint8_t *hash_key, uint32_t salt_len, uint32_t iv_len, CipherAlg cipher_alg, CipherMode cipher_mode, HmacAlg hmac_alg)
{
    etha_ipsec_session_disable(self);
    if (cipher_key != NULL)
    {
        etha_ipsec_reg_write(SEC_SESSION(CIPHER_KEY_LO, self->id), (uint32_t)((uint64_t)(cipher_key)));
        etha_ipsec_reg_write(SEC_SESSION(CIPHER_KEY_HI, self->id), (uint32_t)((uint64_t)(cipher_key) >> 32));
    }
    if (hash_key != NULL)
    {
        etha_ipsec_reg_write(SEC_SESSION(HASH_KEY_LO, self->id), (uint32_t)((uint64_t)(hash_key)));
        etha_ipsec_reg_write(SEC_SESSION(HASH_KEY_HI, self->id), (uint32_t)((uint64_t)(hash_key) >> 32));
    }
    etha_ipsec_reg_write(SEC_SESSION(SALT, self->id), salt);
    etha_ipsec_reg_write(SEC_SESSION(CTX, self->id), SET_XFORM_CTX_SALT_LEN(salt_len) | SET_XFORM_CTX_IV_LEN(iv_len) | SET_XFORM_CTX_CIPHER_ALG(cipher_alg) | SET_XFORM_CTX_CIPHER_MODE(cipher_mode) | SET_XFORM_CTX_HMAC_ALG(hmac_alg));
    etha_ipsec_session_enable(self);
}
void etha_ipsec_session_enable(EthaIpsecSession *self)
{
    etha_ipsec_reg_write(SEC_SESSION(CTX, self->id), etha_ipsec_reg_read(SEC_SESSION(CTX, self->id)) | SET_XFORM_CTX_VALID(1));
}
void etha_ipsec_session_disable(EthaIpsecSession *self)
{
    etha_ipsec_reg_write(SEC_SESSION(CTX, self->id), etha_ipsec_reg_read(SEC_SESSION(CTX, self->id)) & ~SET_XFORM_CTX_VALID(1));
}
#endif