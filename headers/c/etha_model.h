#ifndef __ETHA_MODEL_H__
#define __ETHA_MODEL_H__
#include <stdint.h>
#include <stdbool.h>
typedef void (*etha_irq_handler)(uint32_t id);
typedef enum
{
    ETHA_ENV_LOGGER = 0,
    ETHA_TRACING_REG_ONLY = 1,
    ETHA_TRACING_NO_REG = 2,
    ETHA_TRACING_FULL = 3
} EthaLoggerLvl;

extern void etha_logger_en(EthaLoggerLvl);
extern void etha_logger_dis(void);
extern bool etha_pcap_cmp(const char *lhs, const char *rhs, bool verbose);

extern void etha_simulate_pcap(const char *rx_file, const char *tx_file, int32_t core_id);
extern void etha_simulate_loopback(int32_t core_id);
extern void etha_simulate_raw_socket(const char *socket_file, int32_t core_id);
extern void etha_simulate_tap(const char *tap_file, int32_t core_id);
extern void etha_abort(void);
extern void etha_register_irq_handler(uint32_t id, etha_irq_handler f);
extern void etha_reg_write(uint32_t addr, uint32_t value);
extern uint32_t etha_reg_read(uint32_t addr);

extern void etha_ipsec_simulate(int32_t core_id);
extern void etha_ipsec_abort(void);
extern void etha_ipsec_register_irq_handler(uint32_t id, etha_irq_handler f);
extern void etha_ipsec_reg_write(uint32_t addr, uint32_t value);
extern uint32_t etha_ipsec_reg_read(uint32_t addr);

extern void etha_rohc_simulate(int32_t core_id);
extern void etha_rohc_abort(void);
extern void etha_rohc_register_irq_handler(uint32_t id, etha_irq_handler f);
extern void etha_rohc_reg_write(uint32_t addr, uint32_t value);
extern uint32_t etha_rohc_reg_read(uint32_t addr);
#endif