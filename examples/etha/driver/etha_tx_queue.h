#ifndef __ETHA_TX_QUEUE_H__
#define __ETHA_TX_QUEUE_H__
#include <etha_ring.h>
#include <etha_desc.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    uint32_t id;
    uint32_t resp_ptr;
    EthaRing ring;
} EthaTxQueue;

void etha_tx_queue_init(EthaTxQueue *self, uint32_t id, uint32_t ring_size);
void etha_tx_queue_enable(EthaTxQueue *self);
const TxResultDesc *etha_tx_queue_send(EthaTxQueue *self, uint32_t total_size, const EthaMemBlock *blocks, uint32_t n, bool blocking);
#endif