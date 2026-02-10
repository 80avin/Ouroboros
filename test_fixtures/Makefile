CC_32 = gcc -m32 -O0 -g -fno-pie -no-pie
CC_64 = gcc -O0 -g -fno-pie -no-pie

SOURCES = simple_if simple_loop function_calls nested_control
TARGETS_32 = $(addprefix x86_32/,$(SOURCES))
TARGETS_64 = $(addprefix x86_64/,$(SOURCES))

# Try to build both architectures, but don't fail if 32-bit is unavailable
all: dirs $(TARGETS_64)
	@echo "Building 32-bit binaries (optional)..."
	@$(MAKE) build-32bit || echo "Warning: 32-bit build failed (requires gcc-multilib)"

build-32bit: $(TARGETS_32)

dirs:
	mkdir -p x86_32 x86_64

x86_32/%: %.c
	$(CC_32) $< -o $@

x86_64/%: %.c
	$(CC_64) $< -o $@

clean:
	rm -rf x86_32 x86_64
	rm -f simple_if simple_loop function_calls nested_control

.PHONY: all dirs clean build-32bit
