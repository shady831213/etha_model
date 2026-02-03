// This file is auto generated!
#ifndef __ETHA_SC_DESC_H__
#define __ETHA_SC_DESC_H__
#include <stdint.h>

typedef struct {
    uint32_t addr_lo;
    uint32_t addr_hi;
    uint32_t total_size: 24;
    uint32_t n_blocks: 8;
} __attribute__((packed)) SCFrameDesc;


typedef struct {
    uint64_t addr;
    uint32_t size;
    uint32_t reserved;
} SCBufferEntry;

#endif
