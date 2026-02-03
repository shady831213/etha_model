#ifndef __ETHA_TX_QUEUE_C__
#define __ETHA_TX_QUEUE_C__
#include <etha_tx_queue.h>
#include <etha_regs.h>
#include <etha_model.h>
#include <stdlib.h>
#include <stdio.h>

void etha_tx_queue_init(EthaTxQueue *self, uint32_t id, uint32_t ring_size)
{
    self->id = id;
    self->resp_ptr = 0;
    etha_ring_init(&self->ring, etha_reg_write, etha_reg_read, TX_RING(REQ_BASE_L, id), ring_size, sizeof(TxReqDesc), sizeof(TxResultDesc));
}
void etha_tx_queue_enable(EthaTxQueue *self)
{
    etha_ring_enable(&self->ring);
}

bool etha_tx_ring_resp_valid(EthaTxQueue *self)
{
    uint32_t c_ptr = etha_ring_consumer_ptr(&self->ring);
    if (RING_PTR_ROUND(c_ptr) != RING_PTR_ROUND(self->resp_ptr))
    {
        return RING_PTR_PTR(c_ptr) <= RING_PTR_PTR(self->resp_ptr);
    }
    else
    {
        return RING_PTR_PTR(c_ptr) > RING_PTR_PTR(self->resp_ptr);
    }
}

const TxResultDesc *etha_tx_queue_send(EthaTxQueue *self, uint32_t total_size, const EthaMemBlock *blocks, uint32_t n, bool blocking)
{
    while (!etha_ring_push(&self->ring, TxReqDesc, n, {
        req->frame.addr_lo = (uint32_t)((uint64_t)(blocks[i].addr));
        req->frame.addr_hi = (uint32_t)((uint64_t)(blocks[i].addr) >> 32);
        req->frame.size = blocks[i].size;
        req->frame.start = is_head;
        req->frame.end = is_tail;
        if (is_head)
        {
            req->frame.total_size = total_size;
            req->frame.n_blocks = n - 1;
            req->ctrl.resp_en = blocking;
        }
    }))
        ;
    // printf("before blocking resp_ptr: %x, p_ptr: %x c_ptr: %x\n", self->resp_ptr, etha_ring_producer_ptr(&self->ring), etha_ring_consumer_ptr(&self->ring));
    if (blocking)
    {
        while (!etha_tx_ring_resp_valid(self))
            ;
        const TxResultDesc *ret = (const TxResultDesc *)etha_ring_get_resp(&self->ring, self->resp_ptr);
        self->resp_ptr = etha_ring_producer_ptr(&self->ring);
        // printf("after blocking resp_ptr: %x, p_ptr: %x c_ptr: %x\n", self->resp_ptr, etha_ring_producer_ptr(&self->ring), etha_ring_consumer_ptr(&self->ring));
        return ret;
    }
    else
    {
        self->resp_ptr = etha_ring_producer_ptr(&self->ring);
        return NULL;
    }
}
#endif
