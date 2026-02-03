#ifndef __ETHA_RING_C__
#define __ETHA_RING_C__
#include <etha_ring.h>
#include <etha_ring_regs.h>
#include <etha_model.h>
#include <stdlib.h>

void etha_ring_init(EthaRing *self, etha_ring_reg_write reg_write, etha_ring_reg_read reg_read, uint32_t base, uint32_t ring_size, uint32_t req_size, uint32_t resp_size)
{
    self->reg_write = reg_write;
    self->reg_read = reg_read;
    self->base = base;
    self->ring_size = ring_size;
    self->req_size = req_size;
    self->resp_size = resp_size;
    // NOTE: In real hardware, this must be continuous physical address
    self->req_ring = (void *)aligned_alloc(req_size, ring_size * req_size);
    if (resp_size != 0)
    {
        self->resp_ring = (void *)aligned_alloc(resp_size, ring_size * resp_size);
    }
    else
    {
        self->resp_ring = NULL;
    }
}

void etha_ring_enable(EthaRing *self)
{
    self->reg_write(self->base + RING_REGS_REQ_BASE_L_OFFSET, (uint32_t)((uint64_t)(self->req_ring)));
    self->reg_write(self->base + RING_REGS_REQ_BASE_H_OFFSET, (uint32_t)((uint64_t)(self->req_ring) >> 32));
    if (self->resp_size != 0)
    {
        self->reg_write(self->base + RING_REGS_RESP_BASE_L_OFFSET, (uint32_t)((uint64_t)(self->resp_ring)));
        self->reg_write(self->base + RING_REGS_RESP_BASE_H_OFFSET, (uint32_t)((uint64_t)(self->resp_ring) >> 32));
    }

    self->reg_write(self->base + RING_REGS_SIZE_OFFSET, (uint32_t)(self->ring_size));
    self->reg_write(self->base + RING_REGS_CTRL_OFFSET, SET_RING_CTRL_ENABLE(1));
}

bool etha_ring_full(EthaRing *self)
{
    return RING_STATUS_FULL(self->reg_read(self->base + RING_REGS_STATUS_OFFSET));
}

bool etha_ring_empty(EthaRing *self)
{
    return RING_STATUS_EMPTY(self->reg_read(self->base + RING_REGS_STATUS_OFFSET));
}

bool etha_ring_afull(EthaRing *self)
{
    return RING_STATUS_ALMOST_FULL(self->reg_read(self->base + RING_REGS_STATUS_OFFSET));
}

bool etha_ring_aempty(EthaRing *self)
{
    return RING_STATUS_ALMOST_EMPTY(self->reg_read(self->base + RING_REGS_STATUS_OFFSET));
}

uint32_t etha_ring_incr_ptr(EthaRing *self, uint32_t ptr, uint32_t n)
{
    if (RING_PTR_PTR(ptr) + n > self->ring_size - 1)
    {
        uint32_t round = SET_RING_PTR_ROUND((~RING_PTR_ROUND(ptr)));
        uint32_t ptr_l = RING_PTR_PTR(ptr) + n - self->ring_size;
        return round | ptr_l;
    }
    else
    {
        return ptr + n;
    }
}

uint32_t etha_ring_next_ptr(EthaRing *self, uint32_t ptr)
{
    return etha_ring_incr_ptr(self, ptr, 1);
}

uint32_t etha_ring_c_valids(EthaRing *self)
{
    uint32_t c_ptr = self->reg_read(self->base + RING_REGS_P_CONSUMER_OFFSET);
    uint32_t p_ptr = self->reg_read(self->base + RING_REGS_P_PRODUCER_OFFSET);
    if (RING_PTR_ROUND(c_ptr) != RING_PTR_ROUND(p_ptr))
    {
        return self->ring_size - (RING_PTR_PTR(c_ptr) - RING_PTR_PTR(p_ptr));
    }
    else
    {
        return RING_PTR_PTR(p_ptr) - RING_PTR_PTR(c_ptr);
    }
}

uint32_t etha_ring_resp_valids(EthaRing *self, uint32_t resp_ptr)
{
    uint32_t c_ptr = self->reg_read(self->base + RING_REGS_P_CONSUMER_OFFSET);
    if (RING_PTR_ROUND(c_ptr) != RING_PTR_ROUND(resp_ptr))
    {
        return self->ring_size - (RING_PTR_PTR(resp_ptr) - RING_PTR_PTR(c_ptr));
    }
    else
    {
        return RING_PTR_PTR(c_ptr) - RING_PTR_PTR(resp_ptr);
    }
}

uint32_t etha_ring_p_valids(EthaRing *self)
{
    return self->ring_size - etha_ring_c_valids(self);
}

uint32_t etha_ring_advance_p_n(EthaRing *self, uint32_t n)
{
    uint32_t cur_ptr = etha_ring_producer_ptr(self);
    self->reg_write(self->base + RING_REGS_P_PRODUCER_OFFSET, etha_ring_incr_ptr(self, cur_ptr, n));
    return cur_ptr;
}

uint32_t etha_ring_producer_ptr(EthaRing *self)
{
    return self->reg_read(self->base + RING_REGS_P_PRODUCER_OFFSET);
}

uint32_t etha_ring_consumer_ptr(EthaRing *self)
{
    return self->reg_read(self->base + RING_REGS_P_CONSUMER_OFFSET);
}

uint32_t etha_ring_advance_p(EthaRing *self)
{
    return etha_ring_advance_p_n(self, 1);
}

void *etha_ring_get_req(EthaRing *self, uint32_t ptr)
{
    if (RING_PTR_PTR(ptr) >= self->ring_size)
    {
        return NULL;
    }
    return self->req_ring + (RING_PTR_PTR(ptr) * self->req_size);
}

const void *etha_ring_get_resp(EthaRing *self, uint32_t ptr)
{
    if (RING_PTR_PTR(ptr) >= self->ring_size)
    {
        return NULL;
    }
    return self->resp_ring + (RING_PTR_PTR(ptr) * self->resp_size);
}

#endif