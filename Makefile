DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)/fmi

NETWORK_GRAPH := $(FMI_DIR)/network.gr
NETWORK_CH := $(NETWORK_GRAPH).ch.bincode
NETWORK_HL := $(NETWORK_GRAPH).hl.bincode
NETWORK_TESTS := $(NETWORK_GRAPH).tests.json

NY_GRAPH := $(FMI_DIR)/USA-road-d.NY.gr
NY_CH := $(NY_GRAPH).ch.bincode
NY_HL := $(NY_GRAPH).hl.bincode
NY_TESTS := $(NY_GRAPH).tests.json

STGT_GRAPH := $(FMI_DIR)/stgtregbz.fmi
STGT_QUEUE := $(FMI_DIR)/stgtregbz.que
STGT_SOL := $(FMI_DIR)/stgtregbz.sol

NUM_TESTS := 100000

dirs:
	mkdir $(FMI_DIR)


test_queue_sol:
	cargo run --bin test_dijkstra --release --\
		--graph-path $(STGT_GRAPH)\
		--queue-path $(STGT_QUEUE)\
		--sol-path $(STGT_SOL)

test_ny:
	cargo run --bin test --release --\
		--graph-path $(NY_GRAPH)\
		--ch-path $(NY_CH)\
		--hl-path $(NY_HL)\
		--tests-path $(NY_TESTS)

test:
	cargo run --bin test --release --\
		--graph-path $(NETWORK_GRAPH)\
		--ch-path $(NETWORK_CH)\
		--hl-path $(NETWORK_HL)\
		--tests-path $(NETWORK_TESTS)


create_tests_ny:
	cargo run --bin create_tests --release --\
		--graph-path $(NY_GRAPH)\
		--tests-path $(NY_TESTS)\
		--number-of-tests $(NUM_TESTS)

create_tests:
	cargo run --bin create_tests --release --\
		--graph-path $(NETWORK_GRAPH)\
		--tests-path $(NETWORK_TESTS)\
		--number-of-tests $(NUM_TESTS)


create_ch_ny:
	cargo run --bin create_ch --release --\
		--graph-path $(NY_GRAPH)\
		--ch-graph $(NY_CH)

create_ch:
	cargo run --bin create_ch --release --\
		--graph-path $(NETWORK_GRAPH)\
		--ch-graph $(NETWORK_CH)


compare_ch_ny:
	cargo run --bin compare_ch --release --\
		--graph-path $(NY_GRAPH)\
		--ch-graph $(NY_CH)\
		--tests-path $(NY_TESTS)

compare_ch:
	cargo run --bin compare_ch --release --\
		--graph-path $(NETWORK_GRAPH)\
		--ch-graph $(NETWORK_CH)\
		--tests-path $(NETWORK_TESTS)


test_fast_paths_ny:
	cargo run --bin create_ch_fast_paths --release --\
		--graph-path $(NY_GRAPH)\

test_fast_paths:
	cargo run --bin create_ch_fast_paths --release --\
		--graph-path $(NETWORK_GRAPH)\


test_ch_ny:
	cargo run --bin test_ch --release --\
		--graph $(NY_GRAPH)\
		--ch-graph $(NY_CH)\
		--tests-path $(NY_TESTS)

test_ch:
	cargo run --bin test_ch --release --\
		--graph $(NETWORK_GRAPH)\
		--ch-graph $(NETWORK_CH)\
		--tests-path $(NETWORK_TESTS)


hitting_set_ny:
	cargo run --bin hitting_set --release --\
		--graph-path $(NY_GRAPH)\
		--ch-path $(NY_CH)\
		--hl-path $(NY_HL)\
		--tests-path $(NY_TESTS)

hitting_set:
	cargo run --bin hitting_set --release --\
		--graph-path $(NETWORK_GRAPH)\
		--ch-path $(NETWORK_CH)\
		--hl-path $(NETWORK_HL)\
		--tests-path $(NETWORK_TESTS)


create_hl_ny:
	cargo run --bin create_hl --release --\
		--ch-graph $(NY_CH)\
		--hl-graph $(NY_HL)\

create_hl:
	cargo run --bin create_hl --release --\
		--ch-graph $(NETWORK_CH)\
		--hl-graph $(NETWORK_HL)\


test_hl_ny:
	cargo run --bin test_hl --release --\
		--hl-graph $(NY_HL)\
		--graph-path $(NY_GRAPH)\
		--tests-path $(NY_TESTS)

test_hl:
	cargo run --bin test_hl --release --\
		--hl-graph $(NETWORK_HL)\
		--graph-path $(NETWORK_GRAPH)\
		--tests-path $(NETWORK_TESTS)

