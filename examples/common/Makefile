CFLAGS = -I../../headers/c -I. -std=c11
SRCS = $(wildcard *.c)
OBJS = $(patsubst %.c, %.o, $(SRCS))
$(OBJS): %.o: %.c
	@$(CC) $(CFLAGS) -o $@ -c $< 
	@echo "    CC        $(OBJS)"

clean:
	rm -rf $(OBJS)

.PHONY:clean
