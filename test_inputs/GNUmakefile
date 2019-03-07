TARGET_DIR = $(shell readlink -f $(shell pwd)/../target/test_inputs)
BINS =	${TARGET_DIR}/hello_world/hello_world1 \
	${TARGET_DIR}/hello_world/hello_world2 \
	${TARGET_DIR}/fannkuch_redux/fannkuch_redux

all: ${BINS}

.SUFFIXES:
.SUFFIXES: .ll .s

${TARGET_DIR}/%.s: %.ll
	mkdir -p `dirname $@`
	llc -relocation-model=pic -o $@ $<

${TARGET_DIR}/%: ${TARGET_DIR}/%.s
	clang ${CFLAGS} -o $@ $< ${LDFLAGS}

clean:
	for i in ${BINS}; do rm -f $$i $$i.s; done
