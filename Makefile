DATA_DIR := tests/data
OSM_DIR := $(DATA_DIR)/osm
GEOJSON_DIR := $(DATA_DIR)/test_geojson
FMI_DIR := $(DATA_DIR)/fmi

NETWORK_FMI := $(FMI_DIR)/network.fmi
NETWORK_CONTRACTED:= $(FMI_DIR)/network_contracted.bincode
NETWORK_HUBS:= $(FMI_DIR)/network_hubs.bincode
NETWORK_HUBS_PRUNED:= $(FMI_DIR)/network_hubs_pruned.bincode
NETWORK_TESTS := $(FMI_DIR)/network_tests.json

STGT_FMI := $(FMI_DIR)/stgtregbz.fmi
STGT_CONTRACTED:= $(FMI_DIR)/stgtregbz_contracted.bincode
STGT_HUBS:= $(FMI_DIR)/stgtregbz_hubs.bincode
STGT_HUBS_PRUNED:= $(FMI_DIR)/stgtregbz_hubs_pruned.bincode
STGT_TESTS_JSON := $(FMI_DIR)/stgtregbz_tests.json

NUM_TESTS := 10000
HOP_LIMIT := 3

dirs:
	mkdir tests/data/test_geojson/
	mkdir tests/data/image/
	mkdir tests/data/osm/
	mkdir tests/data/fmi/


test_queue_sol:
	cargo run --bin test_queue_sol --release --\
		--graph-path $(STGT_FMI)\
		--ch-path $(STGT_CONTRACTED)\
		--hl-path $(STGT_HUBS_PRUNED)\
		--queue-path ~/Downloads/Benchs/stgtregbz.que\
		--sol-path ~/Downloads/Benchs/stgtregbz.sol

test:
	cargo run --bin test --release --\
		--graph-path $(STGT_FMI)\
		--ch-path $(STGT_CONTRACTED)\
		--hl-path $(STGT_HUBS_PRUNED)\
		--tests-path $(STGT_TESTS_JSON)


create_tests_stgt:
	cargo run --bin create_tests --release --\
		--graph-path $(STGT_FMI)\
		--tests-path $(STGT_TESTS_JSON)\
		--number-of-tests $(NUM_TESTS)

create_tests:
	cargo run --bin create_tests --release --\
		--graph-path $(NETWORK_FMI)\
		--tests-path $(NETWORK_TESTS)\
		--number-of-tests $(NUM_TESTS)


create_ch_stgt:
	cargo run --bin create_ch --release --\
		--graph-path $(STGT_FMI)\
		--ch-graph $(STGT_CONTRACTED)

create_ch:
	cargo run --bin create_ch --release --\
		--graph-path $(NETWORK_FMI)\
		--ch-graph $(NETWORK_CONTRACTED)


test_ch_stgt:
	cargo run --bin test_ch --release --\
		--ch-graph $(STGT_CONTRACTED)\
		--tests-path $(STGT_TESTS_JSON)

test_ch:
	cargo run --bin test_ch --release --\
		--ch-graph $(NETWORK_CONTRACTED)\
		--tests-path $(NETWORK_TESTS)


create_hl_stgt:
	cargo run --bin create_hl --release --\
		--ch-graph $(STGT_CONTRACTED)\
		--hl-graph $(STGT_HUBS)\
		--hop-limit $(HOP_LIMIT)

create_hl:
	cargo run --bin create_hl --release --\
		--ch-graph $(NETWORK_CONTRACTED)\
		--hl-graph $(NETWORK_HUBS)\
		--hop-limit $(HOP_LIMIT)


test_hl_stgt:
	cargo run --bin test_hl --release --\
		--hl-graph $(STGT_HUBS)\
		--graph-path $(STGT_FMI)\
		--tests-path $(STGT_TESTS_JSON)

test_hl:
	cargo run --bin test_hl --release --\
		--hl-graph $(NETWORK_HUBS)\
		--graph-path $(NETWORK_FMI)\
		--tests-path $(NETWORK_TESTS)

