// This file is auto generated!
#ifndef __ETHA_DESC_H__
#define __ETHA_DESC_H__
#include <stdint.h>

typedef struct {
    uint32_t addr_lo;
    uint32_t addr_hi;
    uint32_t total_size: 24;
    uint32_t n_blocks: 8;
    uint32_t size: 24;
    uint32_t start: 1;
    uint32_t end: 1;
    uint32_t :6;
} __attribute__((packed)) FrameDesc;


typedef struct {
    uint32_t resp_en: 1;
    uint32_t :31;
} __attribute__((packed)) TxCtrlDesc;


typedef struct {
    uint32_t too_large: 1;
    uint32_t too_small: 1;
    uint32_t :30;
    uint32_t padding[1];
} __attribute__((packed)) TxStatusDesc;


typedef struct {
    FrameDesc frame;
    TxCtrlDesc ctrl;
    uint8_t padding[12];
} TxReqDesc;


typedef struct {
    TxStatusDesc status;
} TxResultDesc;


typedef struct {
    uint32_t l2_src_lo;
    uint32_t l2_src_hi: 16;
    uint32_t l2_vlan_flags: 4;
    uint32_t l2_vlan_vid: 12;
    uint32_t l2_dst_lo;
    uint32_t l2_dst_hi: 16;
    uint32_t l2_etype: 16;
    uint32_t l2_header_len: 8;
    uint32_t l2_is_vlan: 1;
    uint32_t :23;
    uint32_t l2_payload_len: 24;
    uint32_t :8;
} __attribute__((packed)) RxResultL2Desc;


typedef struct {
    uint32_t l3_src;
    uint32_t l3_src1;
    uint32_t l3_src2;
    uint32_t l3_src3;
    uint32_t l3_dst;
    uint32_t l3_dst1;
    uint32_t l3_dst2;
    uint32_t l3_dst3;
    uint32_t l3_protocol: 8;
    uint32_t l3_version: 8;
    uint32_t l3_header_len: 16;
    uint32_t l3_payload_len: 24;
    uint32_t :8;
} __attribute__((packed)) RxResultL3Desc;


typedef struct {
    uint32_t l4_src_port: 16;
    uint32_t l4_dst_port: 16;
    uint32_t l4_header_len: 16;
    uint32_t :16;
    uint32_t l4_payload_len: 24;
    uint32_t :8;
} __attribute__((packed)) RxResultL4Desc;


typedef struct {
    uint32_t too_large: 1;
    uint32_t :31;
} __attribute__((packed)) RxStatusDesc;


typedef struct {
    FrameDesc frame;
    RxResultL2Desc l2;
    RxResultL3Desc l3;
    RxResultL4Desc l4;
    RxStatusDesc status;
    uint8_t padding[32];
} RxResultDesc;

#endif
