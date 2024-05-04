DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)

# NETWORK_GRAPH := $(FMI_DIR)/stgtregbz.fmi
# NETWORK_GRAPH := $(FMI_DIR)/network.fmi
# NETWORK_GRAPH := $(FMI_DIR)/aegaeis10-visibility-small.fmi
NETWORK_GRAPH := $(FMI_DIR)/aegaeis10-ref-visibility-mercator.fmi
NETWORK_CH := $(NETWORK_GRAPH).ch.bincode
NETWORK_HL := $(NETWORK_GRAPH).hl.bincode
NETWORK_TESTS_RANDOM := $(NETWORK_GRAPH).tests_random.json
NETWORK_TESTS_DIJKSTRA_RANK := $(NETWORK_GRAPH).tests_dijkstra_rank.json

NY_GRAPH := $(FMI_DIR)/USA-road-d.FLA.gr
# NY_GRAPH := $(FMI_DIR)/bremen_dist.gr
NY_CH := $(NY_GRAPH).ch.bincode
NY_HL := $(NY_GRAPH).hl.bincode
NY_TESTS_RANDOM := $(NY_GRAPH).tests_random.json
NY_TESTS_DIJKSTRA_RANK := $(NY_GRAPH).tests_dijkstra_rank.json

NUM_TESTS := 10000

dirs:
	mkdir $(FMI_DIR)


test_ny:
	cargo run --bin test --release --\
		--graph-path $(NY_GRAPH)\
		--random-pairs $(NY_TESTS_RANDOM)

test:
	cargo run --bin test --release --\
		--graph-path $(NETWORK_GRAPH)\
		--random-pairs $(NETWORK_TESTS_RANDOM)


create_tests_ny:
	cargo run --bin create_tests --release --\
		--graph $(NY_GRAPH)\
		--random-pairs $(NY_TESTS_RANDOM)\
		--dijkstra-rank-pairs $(NY_TESTS_DIJKSTRA_RANK)\
		--number-of-tests $(NUM_TESTS)

create_tests:
	cargo run --bin create_tests --release --\
		--graph $(NETWORK_GRAPH)\
		--random-pairs  $(NETWORK_TESTS_RANDOM)\
		--dijkstra-rank-pairs $(NETWORK_TESTS_DIJKSTRA_RANK)\
		--number-of-tests $(NUM_TESTS)


create_ch_ny:
	cargo run --bin create_ch --release --\
		--infile $(NY_GRAPH)\
		--tests $(NY_TESTS_RANDOM)\
		--outfile $(NY_CH)

create_ch:
	cargo run --bin create_ch --release --\
		--infile $(NETWORK_GRAPH)\
		--tests $(NETWORK_TESTS_RANDOM)\
		--outfile $(NETWORK_CH)



create_tphl_ny:
	cargo run --bin create_top_down_hl --release --\
		--infile $(NY_GRAPH)\
		--tests $(NY_TESTS_RANDOM)\
		--outfile $(NY_HL)

create_tphl:
	cargo run --bin create_top_down_hl  --release --\
		--infile $(NETWORK_GRAPH)\
		--tests $(NETWORK_TESTS_RANDOM)\
		--outfile $(NETWORK_HL)



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
		--contracted-graph $(NY_CH)\
		--tests $(NY_TESTS_RANDOM)\
		--hub-graph $(NY_HL)

create_hl:
	cargo run --bin create_hl --release --\
		--contracted-graph $(NETWORK_CH)\
		--tests $(NETWORK_TESTS_RANDOM)\
		--hub-graph $(NETWORK_HL)


