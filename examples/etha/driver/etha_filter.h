#ifndef __ETHA_FILTER_H__
#define __ETHA_FILTER_H__
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <etha_regs.h>
#include <etha_rx_queue.h>

typedef enum
{
    Blocking = 0,
    Drop = 1,
    Default = 2,
} CongestionAction;

void etha_filter_et(const EthaRxQueue *queue, uint32_t filter_id, CongestionAction congestion_action, uint16_t etype);

void etha_filter_default(const EthaRxQueue *queue, CongestionAction congestion_action);
#endif