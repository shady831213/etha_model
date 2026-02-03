# ehta_model
A ethernet accelerator functional model written in Rust.
A example to show hwo modeling hardware in rust, exploreing hardware device architecture, sw/hw partition,  defineing registers interface and dma descirptors structure..

# How to run

## Build model
```
cargo build --profile release-lto --lib
```

## Build model with rohc
rohc library is a dependency of model with rohc, so rohc library (as a submodule) must be installed. refer to [rohc](https://github.com/didier-barvaux/rohc/blob/master/INSTALL.md).
And the configure could be with '--enable-static ' option.
```
$ ./configure --prefix=/path/to/installation/directory [--enable-static]
$ make all
$ make install
```
```
cargo build --profile release-lto --lib --features='rohc'
```

### Update rohc lib binding
```
cd rohc_bindgen
CLANG_PATH={CLANG_PATH} LIBCLANG_PATH={LIBCLANG_PATH} cargo run -- {ROHC_HEADERS_DIR} {OUTPUT_PATH}
```


## run c example
```
// etha example
cargo build --profile release-lto --lib
cd exmaples/etha
make
./example.exe
```

```
// ipsec example
cargo build --profile release-lto --lib
cd exmaples/etha_ipsec
make
./example.exe
```

```
// rohc example
cargo build --profile release-lto --lib --features='rohc'
cd exmaples/rohc
make
./example.exe
```

## model unit tests
```
cargo test
```

## model unit tests with rohc
```
cargo test --features='rohc'
```

## header_generation
```
cargo run --bin header_gen -- [Options] <OUT_DIR>
//help
cargo run --bin header_gen -- -h
```

## tracing & analysis
### Enable tracing
To enable tracing, please add the following functions.Also can refer to [this example](examples/etha/example.c)
```
...
    //enable env logger, use envvar RUST_LOG
    //enable tracing logger for all event
    etha_logger_en(ETHA_LOGGER_FULL);
...
...
...
    //disable logger and flush all result
    etha_logger_dis();
```
### Tracing result
The tracing result is located in 'model.trace.json' in current working dir. It is in [chrome tracing](https://www.chromium.org/developers/how-tos/trace-event-profiling-tool/) format. Something like:

```
[
{"ph":"M","pid":1,"name":"thread_name","tid":0,"args":{"name":"0"}},
{"ph":"i","pid":1,"ts":842.365,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2093","data":"1024"}},
{"ph":"i","pid":1,"ts":916.437,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2080","data":"40616128"}},
{"ph":"i","pid":1,"ts":923.176,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2081","data":"0"}},
{"ph":"i","pid":1,"ts":925.246,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2082","data":"40662656"}},
{"ph":"i","pid":1,"ts":927.665,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2083","data":"0"}},
{"ph":"i","pid":1,"ts":931.024,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2084","data":"10"}},
{"ph":"i","pid":1,"ts":935.71,"name":"event src/etha/ffi.rs:131","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg write\"","addr":"2092","data":"1"}},
{"ph":"i","pid":1,"ts":945.142,"name":"event src/etha/ffi.rs:147","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg read\"","addr":"2087","data":"0"}},
{"ph":"i","pid":1,"ts":946.656,"name":"event src/etha/ffi.rs:147","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg read\"","addr":"2088","data":"0"}},
{"ph":"i","pid":1,"ts":954.042,"name":"event src/etha/ffi.rs:147","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg read\"","addr":"2088","data":"0"}},
{"ph":"i","pid":1,"ts":955.524,"name":"event src/etha/ffi.rs:147","cat":"etha","tid":0,"s":"t","args":{"name":"\"reg read\"","addr":"2088","data":"0"}},
...
```

### Analysis Tracing result
The result can be visualized in a Chromium/Chrome browser with 'chrome://tracing/'

And there is a python module [tracing_parser.py](python/tracing_parser.py) to help collecting results into python objects to analysis further. This module is executable as a simple demo to summarize some infomations.
```
python3 python/tracing_parser.py {tracing result file}
```
Then it will output something like this
```
------------etha summary begin------------
etha.reg_reads = 187
etha.reg_writes = 67
etha.bus_rd_start_time = 1257.792 us
etha.bus_rd_end_time = 2800.381 us
etha.bus_wr_start_time = 1266.478 us
etha.bus_wr_end_time = 2803.799 us
etha.desc_read_cnt = 72
etha.desc_read_bytes = 1440
etha.desc_write_cnt = 21
etha.desc_write_bytes = 2568
etha.data_read_cnt = 36
etha.data_read_bytes = 27940
etha.data_write_cnt = 36
etha.data_write_bytes = 27940
etha.sc_read_cnt = 0
etha.sc_read_bytes = 0
etha.bus_rd_period = 1542.589 us
etha.bus_wr_period = 1537.321 us
etha.data_rd_throughput = 18112407.12853521 bytes/s
etha.data_wr_throughput = 18174473.64603749 bytes/s
etha.bus_read_cnt = 108
etha.bus_read_bytes = 29380
etha.bus_write_cnt = 57
etha.bus_write_bytes = 30508
etha.bus_wr_throughput = 19844912.02553013 bytes/s
etha.bus_rd_throughput = 19111168.064444575 bytes/s
------------etha summary end------------
```

# featurs
- [x] up to 16 rx and tx rings
- [x] rx and tx from/into pcap files
- [x] etype filters and 5-tuples filters
- [x] parse ethernet/ip/tcp/upd for ingress package
- [x] round-robin arbiter for tx
- [x] tap/raw_socket/loopback for rx and tx(need test in proper enviroment)
- [x] descriptor header generation
- [x] registers header generation
- [x] add pcap_cmp() helper function to compare packages in 2 pcap files.
- [x] support standalone ipsec crypto device.
    - [x] aes-256, aes-128
    - [x] gcm, ccm, cbc, gmac, cbc-mac
    - [x] sha1-hmac, sha256-hmac, sha512_256-hmac,
    - [x] up to 4 queues
    - [x] up to 64 security sessions
    - [x] key caches
- [x] support irqs.
- [x] tracing and analysis.
- [x] support model thread affinity binding.
- [x] support rohc.
    - [x] compress, decompress
    - [x] ROHC_PROFILE_RTP, ROHC_PROFILE_UDP, ROHCv2_PROFILE_IP_UDP_RTP, ROHCv2_PROFILE_IP_UDP
