#ifndef __ETHA_RX_QUEUE_H__
#define __ETHA_RX_QUEUE_H__
#include <etha_ring.h>
#include <etha_desc.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    uint32_t id;
    uint32_t mem_size;
    void *buffer;
    uint32_t resp_ptr;
    EthaRing ring;
} EthaRxQueue;

typedef struct
{
    RxResultDesc *result;
    EthaMemBlock *blocks;
} EthaRxFrame;

void etha_rx_queue_init(EthaRxQueue *self, uint32_t id, uint32_t ring_size, uint32_t mem_size);
void etha_rx_queue_enable(EthaRxQueue *self);
EthaRxFrame etha_rx_queue_receive(EthaRxQueue *self);
void etha_rx_queue_release(EthaRxQueue *self, const EthaRxFrame *frame);
#endif