CFLAGS = -Idriver -I../../headers/c -I../common -std=c11 -pthread -Wl,--no-as-needed -ldl -lm
SRCS = $(wildcard driver/*.c)
SRCS += example.c
OBJS = $(patsubst %.c, %.o, $(SRCS))
COMMON_DIR = ../common
COMMON_SRCS = $(wildcard $(COMMON_DIR)/*.c)
COMMON_OBJS = $(patsubst %.c, %.o, $(COMMON_SRCS))

MODEL = ../../target/release-lto/libetha_model.a lib/x86_64-unknown-linux-gnu/librohc.a
TARGET = example.exe

$(TARGET): $(OBJS) $(MODEL) $(COMMON_OBJS)
	@echo "$(CC) $(CFLAGS) -o $@ $(OBJS) $(MODEL) $(COMMON_OBJS)"
	@$(CC) $(CFLAGS) -o $@ $(OBJS) $(MODEL) $(COMMON_OBJS)
	@echo "    CC    $(notdir $@)"

$(OBJS): %.o: %.c
	@$(CC) $(CFLAGS) -o $@ -c $< 
	@echo "    CC        $(OBJS)"

$(COMMON_OBJS): 
	@make -C $(COMMON_DIR)

clean:
	rm -rf $(OBJS) $(TARGET)
	make -C $(COMMON_DIR) clean

.PHONY:clean
