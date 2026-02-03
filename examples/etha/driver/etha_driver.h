#ifndef __ETHA_DRIVER_H__
#define __ETHA_DRIVER_H__
#include <etha_ring.h>
#include <etha_filter.h>
#include <etha_rx_queue.h>
#include <etha_tx_queue.h>
#include <etha_regs.h>
#include <etha_irqs.h>
#include <etha_model.h>

#define etha_en_rx() etha_reg_write(RX_EN, 1)

#define etha_en_tx() etha_reg_write(TX_EN, 1)
#endif