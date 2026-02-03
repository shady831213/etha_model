// This file is auto generated!
#ifndef __ETHA_IPSEC_DESC_H__
#define __ETHA_IPSEC_DESC_H__
#include <stdint.h>
#include <etha_sc_desc.h>
typedef SCFrameDesc IpsecFrameDesc;

typedef struct {
    uint32_t aad_len: 24;
    uint32_t session_id: 8;
    uint32_t text_len: 24;
    uint32_t encrypt: 1;
    uint32_t resp_en: 1;
    uint32_t aad_copy: 1;
    uint32_t iv_copy: 1;
    uint32_t :4;
} __attribute__((packed)) IpsecFrameCfgDesc;


typedef struct {
    uint32_t aad_offset;
    uint32_t text_offset;
    uint32_t iv_offset;
    uint32_t icv_offset;
} __attribute__((packed)) IpsecFrameFmtDesc;


typedef struct {
    IpsecFrameFmtDesc src;
    IpsecFrameFmtDesc dst;
    IpsecFrameCfgDesc cfg;
} IpsecCfgDesc;


typedef struct {
    SCFrameDesc src;
    SCFrameDesc dst;
    IpsecCfgDesc cfg;
} IpsecReqDesc;


typedef struct {
    uint32_t src_err: 1;
    uint32_t dst_err: 1;
    uint32_t invalid_session: 1;
    uint32_t ciper_err: 1;
    uint32_t auth_fail: 1;
    uint32_t :27;
    uint32_t padding[1];
} __attribute__((packed)) IpsecStatusDesc;


typedef struct {
    IpsecStatusDesc status;
} IpsecResultDesc;

#endif
