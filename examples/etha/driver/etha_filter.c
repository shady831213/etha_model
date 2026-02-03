#ifndef __ETHA_FILTER_C__
#define __ETHA_FILTER_C__
#include <etha_regs.h>
#include <etha_model.h>
#include <etha_rx_queue.h>
#include <etha_filter.h>
#include <assert.h>

void etha_filter_et(const EthaRxQueue *queue, uint32_t filter_id, CongestionAction congestion_action, uint16_t etype)
{
    assert(filter_id < ET_FILTERS_NUM);
    etha_reg_write(ET_FILTER(filter_id), SET_ETHER_TYPE_FILTER_ETYPE((uint32_t)(etype)) | SET_ETHER_TYPE_FILTER_QUEUE_ID(queue->id) | SET_ETHER_TYPE_FILTER_CONGESTION_ACTION((uint32_t)(congestion_action)) | SET_ETHER_TYPE_FILTER_EN(1));
}

void etha_filter_default(const EthaRxQueue *queue, CongestionAction congestion_action)
{
    etha_reg_write(DEFAULT_Q, SET_DEFAULT_QUEUE_QUEUE_ID(queue->id) | SET_DEFAULT_QUEUE_CONGESTION_ACTION((uint32_t)(congestion_action)) | SET_DEFAULT_QUEUE_EN(1));
}
#endif