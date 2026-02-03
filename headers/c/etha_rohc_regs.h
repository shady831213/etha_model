// This file is auto generated!
#ifndef __ETHA_ROHC_REGS_H__
#define __ETHA_ROHC_REGS_H__
#include <stdint.h>
#include <etha_ring_regs.h>

#define QUEUE_NUM 1
#define QUEUE_REGS_OFFSET 0x0
#define RING(name, i) (QUEUE_REGS_OFFSET + (RING_REGS_SIZE * i) + RING_REGS_##name##_OFFSET)

#endif
