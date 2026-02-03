#ifndef __ETHA_RX_QUEUE_C__
#define __ETHA_RX_QUEUE_C__
#include <etha_rx_queue.h>
#include <etha_regs.h>
#include <etha_model.h>
#include <stdlib.h>

void etha_rx_queue_init(EthaRxQueue *self, uint32_t id, uint32_t ring_size, uint32_t mem_size)
{
    self->id = id;
    self->mem_size = mem_size;
    self->buffer = (void *)malloc(mem_size * ring_size);
    self->resp_ptr = 0;
    etha_ring_init(&self->ring, etha_reg_write, etha_reg_read, RX_RING(REQ_BASE_L, id), ring_size, sizeof(uint64_t), sizeof(RxResultDesc));
}

void etha_rx_queue_enable(EthaRxQueue *self)
{
    etha_reg_write(self->ring.base + RING_REGS_MEM_SIZE_OFFSET, self->mem_size);
    etha_ring_enable(&self->ring);
    etha_ring_push(&self->ring, uint64_t, self->ring.ring_size, {
        *req = (uint64_t)(self->buffer) + i * self->mem_size;
    });
}

static uint32_t etha_rx_ring_resp_valids(EthaRxQueue *self)
{
    uint32_t c_ptr = etha_ring_consumer_ptr(&self->ring);
    if (RING_PTR_ROUND(c_ptr) != RING_PTR_ROUND(self->resp_ptr))
    {
        return self->ring.ring_size - (RING_PTR_PTR(self->resp_ptr) - RING_PTR_PTR(c_ptr));
    }
    else
    {
        return RING_PTR_PTR(c_ptr) - RING_PTR_PTR(self->resp_ptr);
    }
}

EthaRxFrame etha_rx_queue_receive(EthaRxQueue *self)
{
    EthaRxFrame frame = {0};
    uint32_t valids = etha_rx_ring_resp_valids(self);
    if (valids > 0)
    {
        frame.result = (RxResultDesc *)etha_ring_get_resp(&self->ring, self->resp_ptr);
        uint32_t n_blocks = (uint32_t)(frame.result->frame.n_blocks) + 1;
        frame.blocks = (EthaMemBlock *)aligned_alloc(sizeof(EthaMemBlock), n_blocks * sizeof(EthaMemBlock));
        for (uint32_t i = 0; i < n_blocks; i++)
        {
            const RxResultDesc *desc = (const RxResultDesc *)etha_ring_get_resp(&self->ring, etha_ring_incr_ptr(&self->ring, self->resp_ptr, i));
            frame.blocks[i].addr = (void *)(((uint64_t)(desc->frame.addr_hi) << 32) | (uint64_t)(desc->frame.addr_lo));
            frame.blocks[i].size = desc->frame.size;
        }
        self->resp_ptr = etha_ring_incr_ptr(&self->ring, self->resp_ptr, frame.result->frame.n_blocks + 1);
    }
    return frame;
}

void etha_rx_queue_release(EthaRxQueue *self, const EthaRxFrame *frame)
{
    if (frame->result != NULL)
    {
        etha_ring_advance_p_n(&self->ring, frame->result->frame.n_blocks + 1);
        free(frame->blocks);
    }
}

#endif