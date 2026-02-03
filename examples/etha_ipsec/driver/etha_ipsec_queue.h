#ifndef __ETHA_IPSEC_QUEUE_H__
#define __ETHA_IPSEC_QUEUE_H__
#include <etha_ring.h>
#include <etha_ipsec_desc.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    uint32_t id;
    EthaRing ring;
} EthaIpsecQueue;

void etha_ipsec_queue_init(EthaIpsecQueue *self, uint32_t id, uint32_t ring_size);
void etha_ipsec_queue_enable(EthaIpsecQueue *self);
uint32_t etha_ipsec_xform_req(EthaIpsecQueue *self, const EthaMemBlock *src, uint32_t src_n, const EthaMemBlock *dst, uint32_t dst_n, const IpsecCfgDesc *cfg);
const IpsecResultDesc *etha_ipsec_xform_resp(EthaIpsecQueue *self, uint32_t ptr);
#endif