#ifndef __ETHA_ROHC_QUEUE_H__
#define __ETHA_ROHC_QUEUE_H__
#include <etha_ring.h>
#include <etha_rohc_desc.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    uint32_t id;
    EthaRing ring;
} EthaRohcQueue;

void etha_rohc_queue_init(EthaRohcQueue *self, uint32_t id, uint32_t ring_size);
void etha_rohc_queue_enable(EthaRohcQueue *self);
uint32_t etha_rohc_xform_req(EthaRohcQueue *self, const EthaMemBlock *src, uint32_t src_n, const EthaMemBlock *dst, uint32_t dst_n, const RohcCfgDesc *cfg);
const RohcResultDesc *etha_rohc_xform_resp(EthaRohcQueue *self, uint32_t ptr);
#endif