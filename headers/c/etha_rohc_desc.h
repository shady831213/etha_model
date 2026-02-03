// This file is auto generated!
#ifndef __ETHA_ROHC_DESC_H__
#define __ETHA_ROHC_DESC_H__
#include <stdint.h>
#include <etha_sc_desc.h>
typedef SCFrameDesc RohcFrameDesc;

typedef struct {
    uint32_t v2: 1;
    uint32_t decomp: 1;
    uint32_t :29;
    uint32_t resp_en: 1;
} __attribute__((packed)) RohcCfgDesc;


typedef struct {
    SCFrameDesc src;
    SCFrameDesc dst;
    RohcCfgDesc cfg;
    uint8_t padding[4];
} RohcReqDesc;


typedef struct {
    uint32_t src_err: 1;
    uint32_t dst_err: 1;
    uint32_t too_small: 1;
    uint32_t bad_crc: 1;
    uint32_t no_ctx: 1;
    uint32_t bad_fmt: 1;
    uint32_t :10;
    uint32_t len: 16;
} __attribute__((packed)) RohcStatusDesc;


typedef struct {
    RohcStatusDesc status;
} RohcResultDesc;

#endif
