#include <stdio.h>
#include <etha_ipsec_driver.h>
#include <etha_model.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

static uint8_t gcm_key[32] = {
    0xdd, 0x73, 0x67, 0x0f,
    0xb2, 0x21, 0xf7, 0xee,
    0x18, 0x5f, 0x58, 0x18,
    0x06, 0x5e, 0x22, 0xdd,
    0xa3, 0x78, 0x0f, 0xc9,
    0x00, 0xfc, 0x02, 0xef,
    0x00, 0x23, 0x2c, 0x66,
    0x1d, 0x7b, 0xff, 0xce};

static uint32_t gcm_salt = 0x53e63dc3;

// testvector from https://github.com/RustCrypto/AEADs/blob/master/aes-gcm/tests/aes256gcm.rs#L1417
int main(int argc, const char *argv[])
{
    // enable env logger, use envvar RUST_LOG
    etha_logger_en(ETHA_TRACING_NO_REG);
    // enable etha ipsec model
    etha_ipsec_simulate(0);

    // config ipsec queues
    EthaIpsecQueue ch;
    etha_ipsec_queue_init(&ch, 0, 10);
    etha_ipsec_queue_enable(&ch);
    printf("config ipsec queues done!\n");

    // config session
    EthaIpsecSession session;
    etha_ipsec_session_init(&session, 0);
    etha_ipsec_session_cfg(&session, (uint8_t *)&gcm_key, gcm_salt, NULL, 4, 8, AES256, GCM, HmacAlgNull);
    printf("config ipsec sessions done!\n");

    uint8_t iv[8] = {
        0x44, 0xcf, 0xbf, 0x22,
        0x8e, 0x16, 0x52, 0xbd};
    uint8_t plaintext[13] = {
        0xad, 0xa4, 0xd9, 0x81,
        0x47, 0xb3, 0x0e, 0x5a,
        0x90, 0x12, 0x29, 0x95,
        0x2a};
    uint8_t ciphertext[13] = {
        0x6e, 0xd4, 0xe4, 0xbd,
        0x1f, 0x95, 0x3d, 0x47,
        0xc5, 0x28, 0x8c, 0x48,
        0xf4};
    uint8_t aad[16] = {
        0xe1, 0xa5, 0xe5, 0x24,
        0x27, 0xf1, 0xc5, 0xb8,
        0x87, 0x57, 0x5a, 0x6f,
        0x2c, 0x44, 0x54, 0x29};
    uint8_t icv[16] = {
        0x40, 0x4e, 0x3a, 0x9b,
        0x9f, 0x5d, 0xda, 0xb9,
        0xee, 0x16, 0x9a, 0x7c,
        0x7c, 0x2c, 0xf7, 0xaf};
    // aad + iv + ciphertext + icv
    uint8_t encrypted_result[16 + 8 + 13 + 16];

    uint32_t start_ptr = etha_ring_producer_ptr(&ch.ring);
    uint32_t end_ptr;
    EthaMemBlock plaintext_buffers[3];
    plaintext_buffers[0].addr = (void *)&aad;
    plaintext_buffers[0].size = sizeof(aad);
    plaintext_buffers[1].addr = (void *)&iv;
    plaintext_buffers[1].size = sizeof(iv);
    plaintext_buffers[2].addr = (void *)&plaintext;
    plaintext_buffers[2].size = sizeof(plaintext);

    EthaMemBlock encrypted_buffers[1];
    encrypted_buffers[0].addr = (void *)&encrypted_result;
    encrypted_buffers[0].size = sizeof(encrypted_result);
    {
        IpsecCfgDesc cfg = {
            .src = {
                .aad_offset = 0,
                .iv_offset = sizeof(aad),
                .text_offset = sizeof(aad) + sizeof(iv)},
            .dst = {.aad_offset = 0, .iv_offset = sizeof(aad), .text_offset = sizeof(aad) + sizeof(iv), .icv_offset = sizeof(aad) + sizeof(iv) + sizeof(ciphertext)},
            .cfg = {
                .aad_len = sizeof(aad),
                .session_id = 0,
                .text_len = sizeof(ciphertext),
                .encrypt = 1,
                .resp_en = 1,
                .aad_copy = 1,
                .iv_copy = 1,
            }};
        end_ptr = etha_ipsec_xform_req(&ch, (EthaMemBlock *)&plaintext_buffers, 3, (EthaMemBlock *)&encrypted_buffers, 1, &cfg);
    }
    // aad + iv + ciphertext
    uint8_t decrypted_result_aad[16] = {0};
    uint8_t decrypted_result_text[8 + 13];
    EthaMemBlock plaintext_buffers_1[2];
    plaintext_buffers_1[0].addr = (void *)&decrypted_result_aad;
    plaintext_buffers_1[0].size = sizeof(decrypted_result_aad);
    plaintext_buffers_1[1].addr = (void *)&decrypted_result_text;
    plaintext_buffers_1[1].size = sizeof(decrypted_result_text);

    {
        IpsecCfgDesc cfg = {
            .src = {
                .aad_offset = 0,
                .iv_offset = sizeof(aad),
                .text_offset = sizeof(aad) + sizeof(iv),
                .icv_offset = sizeof(aad) + sizeof(iv) + sizeof(ciphertext)},
            .dst = {.aad_offset = 0, .iv_offset = sizeof(aad), .text_offset = sizeof(aad) + sizeof(iv)},
            .cfg = {
                .aad_len = sizeof(aad),
                .session_id = 0,
                .text_len = sizeof(ciphertext),
                .encrypt = 0,
                .resp_en = 1,
                .aad_copy = 0,
                .iv_copy = 1,
            }};
        end_ptr = etha_ipsec_xform_req(&ch, (EthaMemBlock *)&encrypted_buffers, 1, (EthaMemBlock *)&plaintext_buffers_1, 2, &cfg);
    }
    while (!etha_ring_empty(&ch.ring))
        ;

    // check encrypted result
    for (int i = 0; i < sizeof(encrypted_result); i++)
    {
        printf("encrypted result[%d] = 0x%0x\n", i, encrypted_result[i]);
    }
    assert(memcmp(&aad, &encrypted_result, sizeof(aad)) == 0);
    assert(memcmp(&iv, &encrypted_result[sizeof(aad)], sizeof(iv)) == 0);
    assert(memcmp(&ciphertext, &encrypted_result[sizeof(aad) + sizeof(iv)], sizeof(ciphertext)) == 0);
    assert(memcmp(&icv, &encrypted_result[sizeof(aad) + sizeof(iv) + sizeof(ciphertext)], sizeof(icv)) == 0);

    // check decrypted result
    for (int i = 0; i < sizeof(decrypted_result_aad); i++)
    {
        printf("decrypted add result[%d] = 0x%0x\n", i, decrypted_result_aad[i]);
        assert(decrypted_result_aad[i] == 0);
    }
    for (int i = 0; i < sizeof(decrypted_result_text); i++)
    {
        printf("decrypted text result[%d] = 0x%0x\n", i, decrypted_result_text[i]);
    }
    assert(memcmp(&iv, &decrypted_result_text, sizeof(iv)) == 0);
    assert(memcmp(&plaintext, &decrypted_result_text[sizeof(iv)], sizeof(plaintext)) == 0);

    // check resp no err
    printf("start ptr %d, end ptr %d\n", start_ptr, end_ptr);
    uint32_t ptr = start_ptr;
    do
    {
        const IpsecResultDesc *resp = etha_ipsec_xform_resp(&ch, ptr);
        printf("resp @ptr %d = 0x%lx\n", ptr, *resp);
        assert((*(uint64_t *)resp) == 0);
        ptr = etha_ring_next_ptr(&ch.ring, ptr);
    } while (ptr != end_ptr);

    etha_ipsec_abort();
    etha_logger_dis();
    printf("test pass!\n");
    return 0;
}