#ifndef __ETHA_ROHC_QUEUE_C__
#define __ETHA_ROHC_QUEUE_C__
#include <etha_rohc_queue.h>
#include <etha_rohc_regs.h>
#include <etha_model.h>
#include <stdlib.h>
#include <stdio.h>
#include <assert.h>

void etha_rohc_queue_init(EthaRohcQueue *self, uint32_t id, uint32_t ring_size)
{
    self->id = id;
    etha_ring_init(&self->ring, etha_rohc_reg_write, etha_rohc_reg_read, RING(REQ_BASE_L, id), ring_size, sizeof(RohcReqDesc), sizeof(RohcResultDesc));
}
void etha_rohc_queue_enable(EthaRohcQueue *self)
{
    etha_ring_enable(&self->ring);
}
static void etha_rohc_frame(RohcFrameDesc *desc, const EthaMemBlock *blocks, uint32_t n)
{
    assert(n != 0);
    if (n == 1)
    {
        desc->addr_lo = (uint32_t)((uint64_t)(blocks[0].addr));
        desc->addr_hi = (uint32_t)((uint64_t)(blocks[0].addr) >> 32);
        desc->total_size = blocks[0].size;
        desc->n_blocks = 0;
    }
    else
    {
        SCBufferEntry *sc_list = (SCBufferEntry *)(blocks);
        int total_size = 0;
        for (int i = 0; i < n; i++)
        {
            total_size += sc_list[i].size;
        }
        desc->addr_lo = (uint32_t)((uint64_t)(sc_list));
        desc->addr_hi = (uint32_t)((uint64_t)(sc_list) >> 32);
        desc->total_size = total_size;
        desc->n_blocks = n - 1;
    }
}

uint32_t etha_rohc_xform_req(EthaRohcQueue *self, const EthaMemBlock *src, uint32_t src_n, const EthaMemBlock *dst, uint32_t dst_n, const RohcCfgDesc *cfg)
{
    while (!etha_ring_push(&self->ring, RohcReqDesc, 1, {
        etha_rohc_frame(&req->src, src, src_n);
        etha_rohc_frame(&req->dst, dst, dst_n);
        req->cfg = *cfg;
    }))
        ;
    return etha_ring_producer_ptr(&self->ring);
}
const RohcResultDesc *etha_rohc_xform_resp(EthaRohcQueue *self, uint32_t ptr)
{
    return (RohcResultDesc *)etha_ring_get_resp(&self->ring, ptr);
}
#endif
