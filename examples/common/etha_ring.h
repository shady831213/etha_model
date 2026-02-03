#ifndef __ETHA_RING_H__
#define __ETHA_RING_H__
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
typedef void (*etha_ring_reg_write)(uint32_t addr, uint32_t value);
typedef uint32_t (*etha_ring_reg_read)(uint32_t addr);
typedef struct
{
    uint32_t base;
    uint32_t ring_size;
    uint32_t req_size;
    uint32_t resp_size;
    void *req_ring;
    void *resp_ring;
    etha_ring_reg_write reg_write;
    etha_ring_reg_read reg_read;
} EthaRing;

typedef struct
{
    void *addr;
    uint32_t size;
    uint32_t reserved;
} EthaMemBlock;

void etha_ring_init(EthaRing *self, etha_ring_reg_write reg_write, etha_ring_reg_read reg_read, uint32_t base, uint32_t ring_size, uint32_t req_size, uint32_t resp_size);

void etha_ring_enable(EthaRing *self);

bool etha_ring_full(EthaRing *self);

bool etha_ring_empty(EthaRing *self);

bool etha_ring_afull(EthaRing *self);

bool etha_ring_aempty(EthaRing *self);

uint32_t etha_ring_incr_ptr(EthaRing *self, uint32_t ptr, uint32_t n);

uint32_t etha_ring_next_ptr(EthaRing *self, uint32_t ptr);

uint32_t etha_ring_c_valids(EthaRing *self);

uint32_t etha_ring_p_valids(EthaRing *self);

uint32_t etha_ring_advance_p_n(EthaRing *self, uint32_t n);

uint32_t etha_ring_producer_ptr(EthaRing *self);

uint32_t etha_ring_consumer_ptr(EthaRing *self);

uint32_t etha_ring_advance_p(EthaRing *self);

void *etha_ring_get_req(EthaRing *self, uint32_t ptr);

const void *etha_ring_get_resp(EthaRing *self, uint32_t ptr);

#define etha_ring_push(ring, req_type, n, ...) ({                                                      \
    bool ret;                                                                                          \
    if (etha_ring_p_valids((ring)) < n)                                                                \
    {                                                                                                  \
        ret = false;                                                                                   \
    }                                                                                                  \
    else                                                                                               \
    {                                                                                                  \
        uint32_t ptr = (ring)->reg_read((ring)->base + RING_REGS_P_PRODUCER_OFFSET);                   \
        for (uint32_t i = 0; i < (n); i++)                                                             \
        {                                                                                              \
            req_type *req = (req_type *)etha_ring_get_req((ring), etha_ring_incr_ptr((ring), ptr, i)); \
            bool is_head = i == 0;                                                                     \
            bool is_tail = i == (n) - 1;                                                               \
            do                                                                                         \
            {                                                                                          \
                __VA_ARGS__                                                                            \
            } while (0);                                                                               \
        }                                                                                              \
        etha_ring_advance_p_n((ring), (n));                                                            \
        ret = true;                                                                                    \
    }                                                                                                  \
    ret;                                                                                               \
})

#endif