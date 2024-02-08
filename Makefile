DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)/fmi

NETWORK_GRAPH := $(FMI_DIR)/network.fmi
NETWORK_CH:= $(FMI_DIR)/network_contracted.bincode
NETWORK_HL:= $(FMI_DIR)/network_hubs.bincode
NETWORK_TESTS := $(FMI_DIR)/network_tests.json

NY_GRAPH := $(FMI_DIR)/USA-road-d.NY.gr
NY_CH:= $(FMI_DIR)/stgtregbz_contracted.bincode
NY_HL:= $(FMI_DIR)/stgtregbz_hubs.bincode
NY_TESTS := $(FMI_DIR)/stgtregbz_tests.json

NUM_TESTS := 10000
HOP_LIMIT := 3

dirs:
	mkdir tests/data/fmi/


test_queue_sol:
	cargo run --bin test_queue_sol --release --\
		--graph-path $(NY_GRAPH)\
		--ch-path $(NY_CH)\
		--hl-path $(NY_HL)\
		--queue-path ~/Downloads/Benchs/stgtregbz.que\
		--sol-path ~/Downloads/Benchs/stgtregbz.sol

test:
	cargo run --bin test --release --\
		--graph-path $(NY_GRAPH)\
		--ch-path $(NY_CH)\
		--hl-path $(NY_HL)\
		--tests-path $(NY_TESTS)


create_tests_stgt:
	cargo run --bin create_tests --release --\
		--graph-path $(NY_GRAPH)\
		--tests-path $(NY_TESTS)\
		--number-of-tests $(NUM_TESTS)

create_tests:
	cargo run --bin create_tests --release --\
		--graph-path $(NETWORK_GRAPH)\
		--tests-path $(NETWORK_TESTS)\
		--number-of-tests $(NUM_TESTS)


create_ch_stgt:
	cargo run --bin create_ch --release --\
		--graph-path $(NY_GRAPH)\
		--ch-graph $(NY_CH)

create_ch:
	cargo run --bin create_ch --release --\
		--graph-path $(NETWORK_GRAPH)\
		--ch-graph $(NETWORK_CH)


test_ch_stgt:
	cargo run --bin test_ch --release --\
		--ch-graph $(NY_CH)\
		--tests-path $(NY_TESTS)

test_ch:
	cargo run --bin test_ch --release --\
		--ch-graph $(NETWORK_CH)\
		--tests-path $(NETWORK_TESTS)


create_hl_stgt:
	cargo run --bin create_hl --release --\
		--ch-graph $(NY_CH)\
		--hl-graph $(NY_HL)\
		--hop-limit $(HOP_LIMIT)

create_hl:
	cargo run --bin create_hl --release --\
		--ch-graph $(NETWORK_CH)\
		--hl-graph $(NETWORK_HL)\
		--hop-limit $(HOP_LIMIT)


test_hl_stgt:
	cargo run --bin test_hl --release --\
		--hl-graph $(NY_HL)\
		--graph-path $(NY_GRAPH)\
		--tests-path $(NY_TESTS)

test_hl:
	cargo run --bin test_hl --release --\
		--hl-graph $(NETWORK_HL)\
		--graph-path $(NETWORK_GRAPH)\
		--tests-path $(NETWORK_TESTS)

