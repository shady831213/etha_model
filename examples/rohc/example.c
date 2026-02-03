#include <stdio.h>
#include <etha_rohc_driver.h>
#include <etha_model.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

int main(int argc, const char *argv[])
{
    // enable etha rohc model
    etha_rohc_simulate(0);

    // config rohc queues
    EthaRohcQueue ch;
    etha_rohc_queue_init(&ch, 0, 10);
    etha_rohc_queue_enable(&ch);
    printf("config rohc queues done!\n");

    uint8_t buf[] =
        {
            0x45, 0x00, 0x00, 0x54, 0x00, 0x00, 0x40, 0x00,
            0x40, 0x01, 0x93, 0x52, 0xc0, 0xa8, 0x13, 0x01,
            0xc0, 0xa8, 0x13, 0x05, 0x08, 0x00, 0xe9, 0xc2,
            0x9b, 0x42, 0x00, 0x01, 0x66, 0x15, 0xa6, 0x45,
            0x77, 0x9b, 0x04, 0x00, 0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13,
            0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23,
            0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b,
            0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
            0x34, 0x35, 0x36, 0x37};

    uint8_t comp_result[100] = {0};

    uint32_t start_ptr = etha_ring_producer_ptr(&ch.ring);
    uint32_t end_ptr;
    EthaMemBlock pkt;
    pkt.addr = (void *)&buf;
    pkt.size = sizeof(buf);

    EthaMemBlock comp_pkt;
    comp_pkt.addr = (void *)&comp_result;
    comp_pkt.size = sizeof(comp_result);

    {
        RohcCfgDesc cfg = {
            .v2 = 0,
            .decomp = 0,
            .resp_en = 1};
        end_ptr = etha_rohc_xform_req(&ch, (EthaMemBlock *)&pkt, 1, (EthaMemBlock *)&comp_pkt, 1, &cfg);
    }
    while (!etha_ring_empty(&ch.ring))
        ;

    // check resp no err
    printf("start ptr %d, end ptr %d\n", start_ptr, end_ptr);
    uint32_t ptr = start_ptr;
    do
    {
        const RohcResultDesc *resp = etha_rohc_xform_resp(&ch, ptr);
        printf("resp @ptr %d = 0x%lx\n", ptr, *resp);
        assert((*(uint16_t *)resp) == 0);
        printf("len = %d\n", resp->status.len);
        assert(resp->status.len == 85);
        ptr = etha_ring_next_ptr(&ch.ring, ptr);
    } while (ptr != end_ptr);

    etha_rohc_abort();
    printf("test pass!\n");
    return 0;
}