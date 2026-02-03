#include <stdio.h>
#include <etha_driver.h>
#include <etha_model.h>
#include <stdlib.h>

void tx_full_handler(uint32_t id)
{
    printf("get irq id %d!\n", id);
}

int main(int argc, const char *argv[])
{
    // enable env logger, use envvar RUST_LOG
    // enable tracing logger for all event
    etha_logger_en(ETHA_TRACING_FULL);
    // enable etha model
    etha_simulate_pcap("../../pcaps/ecpri_20_pkts.pcap", "../../pcaps/tmp/example.pcap", -1);

    // config 2 rx queues
    EthaRxQueue default_q, ecpri_q;
    etha_rx_queue_init(&default_q, 1, 10, 1024);
    etha_rx_queue_enable(&default_q);
    etha_filter_default(&default_q, Blocking);

    etha_rx_queue_init(&ecpri_q, 0, 10, 1060);
    etha_rx_queue_enable(&ecpri_q);
    etha_filter_et(&ecpri_q, 0, Blocking, 0xaefe);
    printf("config rx queues done!\n");

    // config 1 tx queues
    EthaTxQueue tx_q;
    etha_tx_queue_init(&tx_q, 0, 2);
    etha_tx_queue_enable(&tx_q);
    printf("config tx queues done!\n");

    // open tx_q full interrupt
    etha_reg_write(TX_RING(INT_MASK, 0), SET_RING_STATUS_FULL(1));
    // register handler
    etha_register_irq_handler(EthaTxChIrq0, tx_full_handler);

    // enable rx and tx
    etha_en_rx();
    etha_en_tx();

    // loopback 20 packages
    int cnt = 0;
    while (cnt < 20)
    {
        EthaRxFrame frame = etha_rx_queue_receive(&ecpri_q);
        if (frame.result != NULL)
        {
            uint32_t n_blocks = (uint32_t)(frame.result->frame.n_blocks) + 1;
            printf("receive one Ecpri package! len = %d, etype= 0x%x, n_blocks=%d\n", frame.result->frame.total_size, frame.result->l2.l2_etype, n_blocks);
            cnt++;
            if (cnt < 20)
            {
                etha_tx_queue_send(&tx_q, frame.result->frame.total_size, frame.blocks, n_blocks, false);
            }
            else
            {
                etha_tx_queue_send(&tx_q, frame.result->frame.total_size, frame.blocks, n_blocks, true);
                printf("all package done!, cnt = %d\n", cnt);
            }
            etha_rx_queue_release(&ecpri_q, &frame);
        }
    }
    etha_abort();
    etha_logger_dis();
    return etha_pcap_cmp("../../pcaps/ecpri_20_pkts.pcap", "../../pcaps/tmp/example.pcap", true);
}